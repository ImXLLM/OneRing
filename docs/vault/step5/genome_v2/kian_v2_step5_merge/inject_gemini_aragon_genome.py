#!/usr/bin/env python3
"""Idempotent KbEntry injector. Requires an explicit existing DB path; never creates one silently."""
import argparse, json, sqlite3
from pathlib import Path
MODEL_KEY='kb:genome-model-gemini-4-aragon-high'
INDEX_KEY='kb:arch-omni-model-genome-9file-engine'
MODEL_VALUE={"model_id":"gemini-4-aragon-high","profile_path":"OMNI-MODEL-GENOME/models/gemini-4-aragon-high/","score_status":"user-supplied qualitative estimates; not live-model verified","active":True}
INDEX_VALUE={"active_models":["gpt-6-standard-free","gemini-4-aragon-high"],"profile_count":2,"verification_status":"artifact probes reproducible; no live Gemini endpoint verification","merge_step":"KIAN v2 Step 5"}

def upsert(conn, table, key, value):
    q=lambda s:'"'+s.replace('"','""')+'"'
    cols=conn.execute(f'PRAGMA table_info({q(table)})').fetchall()
    if not cols: raise RuntimeError(f'table {table!r} does not exist')
    names=[r[1] for r in cols]; keycol=next((n for n in ('id','key') if n in names),None)
    if not keycol: raise RuntimeError(f'{table}: neither id nor key primary selector exists; columns={names}')
    valcol=next((n for n in ('value','content','body','text','data','json') if n in names and n!=keycol),None)
    if not valcol: raise RuntimeError(f'{table}: no recognized value/content column; columns={names}')
    payload=json.dumps(value,ensure_ascii=False,sort_keys=True)
    existing=conn.execute(f'SELECT 1 FROM {q(table)} WHERE {q(keycol)}=?',(key,)).fetchone()
    if existing:
        conn.execute(f'UPDATE {q(table)} SET {q(valcol)}=? WHERE {q(keycol)}=?',(payload,key)); action='updated'
    else:
        insert={keycol:key,valcol:payload}
        # Populate other NOT NULL columns that have no default conservatively.
        for row in cols:
            name=row[1]; notnull=bool(row[3]); default=row[4]; pk=bool(row[5])
            if name not in insert and notnull and default is None and not pk:
                if name in ('createdAt','created_at','updatedAt','updated_at'): insert[name]=''
                elif row[2].upper() in ('TEXT','VARCHAR','CHAR'): insert[name]=''
                elif 'INT' in row[2].upper() or 'REAL' in row[2].upper(): insert[name]=0
                else: raise RuntimeError(f'cannot safely synthesize required column {name!r} ({row[2]})')
        sql=f'INSERT INTO {q(table)} ({", ".join(q(n) for n in insert)}) VALUES ({", ".join("?" for _ in insert)})'
        conn.execute(sql,tuple(insert.values())); action='inserted'
    return {'table':table,'key_column':keycol,'value_column':valcol,'action':action,'key':key}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('database',nargs='?',default='seed/kian-production.db'); ap.add_argument('--table',default='KbEntry'); ap.add_argument('--dry-run',action='store_true'); a=ap.parse_args()
    path=Path(a.database)
    if not path.is_file(): raise SystemExit(f'Database does not exist: {path}; no database was created.')
    conn=sqlite3.connect(path)
    try:
        conn.execute('BEGIN IMMEDIATE')
        outcomes=[upsert(conn,a.table,MODEL_KEY,MODEL_VALUE),upsert(conn,a.table,INDEX_KEY,INDEX_VALUE)]
        if a.dry_run: conn.rollback()
        else: conn.commit()
        print(json.dumps({'dry_run':a.dry_run,'results':outcomes},indent=2))
    except Exception:
        conn.rollback(); raise
    finally: conn.close()
if __name__=='__main__': main()
