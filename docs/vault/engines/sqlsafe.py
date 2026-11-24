"""sqlsafe — مرکزیت اعتبارسنجی شناسه‌های داینامیک SQL (BUG-02/06/07)

هر جایی که نام ستون/ترتیب از ورودی یا نگاشت داخلی به رشتهٔ SQL می‌رود،
باید از این ماژول عبور کند. هیچ f-string مستقیمی نباید شناسهٔ SQL بسازد.
"""
import re

_IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")

# ستون‌های مجاز به‌ازای جدول — تنها منبع حقیقت برای SQL داینامیک
PRODUCT_IMG_COLS = frozenset({"img_main", "img_feed", "img_story"})
PRODUCT_EDITABLE_COLS = frozenset({
    "title", "description", "price_credits", "photo_path",
    "file_path", "file_type", "tags", "link",
})
PRODUCT_FLAG_COLS = frozenset({"is_active", "is_featured"})
ENGAGEMENT_COUNT_COLS = frozenset({
    "like_count", "dislike_count", "save_count", "views", "clicks", "impressions",
})
USER_PROFILE_COLS = frozenset({"avatar_path", "cover_path"})
USERS_ORDER = {
    "rich": "credits DESC",
    "new": "user_id DESC",
    "id": "user_id ASC",
}


class UnsafeIdentifier(ValueError):
    """شناسهٔ SQL غیرمجاز — همیشه fail-fast، هرگز بی‌صدا."""


def _validate(name: str) -> str:
    if not isinstance(name, str) or not _IDENT.match(name):
        raise UnsafeIdentifier(f"invalid SQL identifier: {name!r}")
    return name


def col(table: str, name: str, allowed: frozenset) -> str:
    """نام ستون را فقط اگر در whitelist جدول باشد برمی‌گرداند."""
    _validate(table)
    _validate(name)
    if name not in allowed:
        raise UnsafeIdentifier(f"column {name!r} not allowed on {table}")
    return name


def order(table: str, mode: str, mapping: dict, default: str) -> str:
    """عبارت ORDER BY فقط از نگاشت ثابت — هیچ ورودی خامی وارد SQL نمی‌شود."""
    _validate(table)
    expr = mapping.get(mode, default)
    if expr not in mapping.values():
        raise UnsafeIdentifier(f"order expr {expr!r} not whitelisted for {table}")
    return expr


def int_csv(values) -> str:
    """فهرست صحیح را به رشتهٔ امن IN (...) تبدیل می‌کند؛ هر عنصر غیرعددی خطا."""
    out = []
    for v in values:
        if isinstance(v, bool) or not isinstance(v, int):
            try:
                iv = int(str(v))
            except (TypeError, ValueError):
                raise UnsafeIdentifier(f"non-integer IN member: {v!r}")
        else:
            iv = v
        out.append(str(iv))
    return ",".join(out)
