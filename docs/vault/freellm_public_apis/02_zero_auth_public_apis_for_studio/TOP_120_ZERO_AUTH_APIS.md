# ۱۲۰ API عمومی برتر بدون کلید (`Auth: No` + `HTTPS: Yes`) برای KIAN Studio
- کل APIهای بدون کلید و HTTPS در `public-apis`: **1150** (شامل **665** مورد با `CORS: Yes`)
- منتخبان KIAN Studio: **120** مورد (**115** مورد `CORS: Yes` + موارد ضروری مرورگری مثل Wikipedia / HackerNews / Free Dictionary / Open Trivia)

## الگوهای آماده `fetch()` برای اپلیکیشن‌های تک‌فایلی `<!DOCTYPE html>`
```javascript
// Open-Meteo
const res = await fetch('https://api.open-meteo.com/v1/forecast?latitude=35.6892&longitude=51.3890&current_weather=true&hourly=temperature_2m'); const data = await res.json();

// Frankfurter
const res = await fetch('https://api.frankfurter.app/latest?from=USD&to=EUR,GBP,JPY'); const data = await res.json();

// Currency-api
const res = await fetch('https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json'); const data = await res.json();

// CoinGecko
const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=bitcoin,ethereum,solana&vs_currencies=usd&include_24hr_change=true'); const data = await res.json();

// REST Countries
const res = await fetch('https://restcountries.com/v3.1/name/iran?fields=name,capital,population,flags,currencies'); const data = await res.json();

// Country
const res = await fetch('https://api.country.is/'); const { ip, country } = await res.json();

// GeoJS
const res = await fetch('https://get.geojs.io/v1/ip/geo.json'); const geo = await res.json();

// HackerNews
const ids = await (await fetch('https://hacker-news.firebaseio.com/v0/topstories.json')).json(); const top5 = await Promise.all(ids.slice(0,5).map(id => fetch(`https://hacker-news.firebaseio.com/v0/item/${id}.json`).then(r=>r.json())));

// Wikipedia
const res = await fetch('https://en.wikipedia.org/api/rest_v1/page/summary/Artificial_intelligence'); const data = await res.json();

// Free Dictionary
const res = await fetch('https://api.dictionaryapi.dev/api/v2/entries/en/serendipity'); const data = await res.json();

// DummyJSON
const res = await fetch('https://dummyjson.com/products?limit=12'); const { products } = await res.json();

// Open Trivia
const res = await fetch('https://opentdb.com/api.php?amount=10&type=multiple'); const { results } = await res.json();

// Spaceflight News
const res = await fetch('https://api.spaceflightnewsapi.net/v4/articles/?limit=10'); const { results } = await res.json();

// JokeAPI
const res = await fetch('https://v2.jokeapi.dev/joke/Programming?safe-mode'); const joke = await res.json();

// Iconify
const res = await fetch('https://api.iconify.design/search?query=rocket&limit=20'); const { icons } = await res.json();

// Art Institute of Chicago
const res = await fetch('https://api.artic.edu/api/v1/artworks?limit=10&fields=id,title,artist_display,image_id'); const { data } = await res.json();

// PoetryDB
const res = await fetch('https://poetrydb.org/random/3'); const poems = await res.json();

// PokéAPI (GraphQL)
const res = await fetch('https://pokeapi.co/api/v2/pokemon?limit=20'); const { results } = await res.json();

// CheapShark
const res = await fetch('https://www.cheapshark.com/api/1.0/deals?storeID=1&upperPrice=15'); const deals = await res.json();

// balldontlie
const res = await fetch('https://api.balldontlie.io/v1/teams');

```

## فهرست کامل ۱۲۰ API منتخب
| # | Category | API | CORS | URL | Description |
|---|---|---|---|---|---|
| 1 | Animals | Cat Facts | Yes | https://catfact.ninja/ | Random cat facts |
| 2 | Animals | HTTP Cat | Yes | https://http.cat/ | Cat for every HTTP Status |
| 3 | Art & Design | Art Institute of Chicago | Yes | https://api.artic.edu/docs/ | Art |
| 4 | Art & Design | EmojiHub | Yes | https://github.com/cheatsnake/emojihub | Get emojis by categories and groups |
| 5 | Art & Design | Favicondl | Yes | https://favicondl.com/documentation.html | Get any website's favicon by URL (SVG/PNG/ICO), with JSON metadata and fallbacks |
| 6 | Art & Design | Iconify | Yes | https://iconify.design/docs/api/ | Search and fetch SVG icons from 200+ open source icon sets |
| 7 | Books | PoetryDB | Yes | https://github.com/thundercomb/poetrydb#readme | Enables you to get instant data from our vast poetry collection |
| 8 | Calendar | HolidayExplore | Yes | https://holidayexplore.com/api-docs/ | Global public holidays and bank closures |
| 9 | Calendar | SalahTable | Yes | https://salahtable.com/prayer-times-api | Islamic prayer times for 400+ cities, by day or month, with six calculation methods |
| 10 | Calendar | caldays | Yes | https://caldays.com/api | Public holidays for 195+ countries |
| 11 | Cryptocurrency | 0x | Yes | https://docs.0x.org/ | API for querying token and pool stats across various liquidity pools |
| 12 | Cryptocurrency | BTCGlobe | Yes | https://btcglobe.live/join#makers | Bitcoin height, fees, mempool and peer data served live from two self-hosted nodes |
| 13 | Cryptocurrency | Bitcoin Halving | Yes | https://why21million.com/halving-api/ | Halving era, block reward, and schedule arithmetic for any Bitcoin block height |
| 14 | Cryptocurrency | BitcoinYield | Yes | https://bitcoinyield.co/docs | Live yields on BTC, ETH and stablecoins across DeFi and CeFi, with an A-D risk grade per offer |
| 15 | Cryptocurrency | BlazePhoenix | Yes | https://blazephoenix.xyz/api | On-chain DEX aggregator quotes and route execution data |
| 16 | Cryptocurrency | CoinGecko | Yes | http://www.coingecko.com/api | Cryptocurrency Price, Market, and Developer/Social Data |
| 17 | Currency Exchange | AllRatesToday | Yes | https://allratestoday.com/central-bank-rates-api/ | Official exchange rates published by 121 central banks and tax authorities, keyless latest table |
| 18 | Currency Exchange | Currency-api | Yes | https://github.com/fawazahmed0/currency-api#readme | Free Currency Exchange Rates API with 150+ Currencies & No Rate Limits |
| 19 | Currency Exchange | Frankfurter | Yes | https://frankfurter.dev | Exchange rates, currency conversion and time series |
| 20 | Currency Exchange | Quotelinks Exchange Rates | Yes | https://www.quotelinks.com/api/exchange-rates/ | ECB daily euro reference rates for 30 currencies since 1999, with conversion; Frankfurter v1 format |
| 21 | Currency Exchange | api-mint | Yes | https://api-mint.hoiwan.workers.dev | Free utility APIs: timezone, forex rates, crypto prices, page metadata |
| 22 | Development | AI SENSE | Yes | https://aisense.no/free-public-apis | Utility endpoints: time, hashing, encoding, QR, JSON, images, webhooks, temporary storage |
| 23 | Development | APIzone | Yes | https://apizone.io/api-docs | Independent uptime & status monitoring for 200+ popular APIs |
| 24 | Development | Agify.io | Yes | https://agify.io | Estimates the age from a first name |
| 25 | Development | Beeceptor | Yes | https://beeceptor.com/ | Build a mock Rest API endpoint in seconds |
| 26 | Development | FreeQrly | Yes | https://freeqrly.com/developers/ | Generate static QR codes (URL, WiFi, vCard, text, geo) as SVG or PNG. Free, no API key |
| 27 | Development | Quotable | Yes | https://github.com/lukePeavey/quotable | Random quotes with authors and tags |
| 28 | Development | The Networkery | Yes | https://thenetworkery.com/api | DNS, email authentication, TLS, BGP/RPKI, WHOIS and subnet checks with plain-English findings |
| 29 | Dictionaries | Random Lexicon | Yes | https://randomlexicon.com/api-reference | Random words and definitions using optional filters |
| 30 | Dictionaries | Wiktionary | Yes | https://en.wiktionary.org/w/api.php | Collaborative dictionary data |
| 31 | Dictionaries | WordSoHard | Yes | https://wordsohard.com/api | Word definitions with Scrabble validity and tile score in one call, built for word games |
| 32 | Entertainment | Fun Fact | Yes | https://api.aakhilv.me | A simple HTTPS api that can randomly select and return a fact from the FFA database |
| 33 | Entertainment | JokeAPI | Yes | https://v2.jokeapi.dev/ | Programmable jokes in multiple languages with filtering options |
| 34 | Entertainment | PotterDB | Yes | https://docs.potterdb.com/ | Harry Potter universe database with characters, spells, potions and more |
| 35 | Environment | Open-Meteo | Yes | https://open-meteo.com/ | Free weather API for non-commercial use |
| 36 | Environment | SpotWave | Yes | https://spotwave.eu/api-docs | Day-ahead electricity prices for 40+ European market zones, incl. taxes and VAT |
| 37 | Finance | CardBinCheck | Yes | https://cardbincheck.com/bin-lookup-api | Card BIN/IIN lookup: network, issuer, country and card type for the first 6-8 digits |
| 38 | Finance | Econdb | Yes | https://www.econdb.com/api/ | Global macroeconomic data |
| 39 | Finance | EstimateTax | Yes | https://estimatetax.net/api/ | US federal, state and local income tax calculation for 2026, with the source of every figure |
| 40 | Food & Drink | Open Brewery DB | Yes | https://www.openbrewerydb.org | Breweries, Cideries and Craft Beer Bottle Shops |
| 41 | Games & Comics | CheapShark | Yes | https://www.cheapshark.com/api | Steam/PC Game Prices and Deals |
| 42 | Games & Comics | Disney | Yes | https://disneyapi.dev | Information of Disney characters |
| 43 | Games & Comics | PokéAPI (GraphQL) | Yes | https://github.com/mazipan/graphql-pokeapi | The Unofficial GraphQL for PokeAPI |
| 44 | Geocoding | 17NAS | Yes | https://17nas.com | Free IP geolocation API with IPv4/IPv6 detection, ASN and country info, no auth required |
| 45 | Geocoding | Country | Yes | http://country.is/ | Get your visitor's country from their IP |
| 46 | Geocoding | GeoJS | Yes | https://www.geojs.io/ | IP geolocation with ChatOps integration |
| 47 | Geocoding | GeographQL | Yes | https://geographql.netlify.app | A Country, State, and City GraphQL API |
| 48 | Geocoding | HackMyIP | Yes | https://hackmyip.com/api | IP geolocation, ISP and privacy/VPN scoring, email breach checks, DNS and WHOIS lookups |
| 49 | Geocoding | REST Countries | Yes | https://restcountries.com | Get information about countries via a RESTful API |
| 50 | Music | ListenBrainz | Yes | https://listenbrainz.readthedocs.io/en/latest/users/api/index.html | Open music listening data: scrobble history, statistics and recommendations |
| 51 | Music | Radio Browser | Yes | https://api.radio-browser.info/ | List of internet radio stations |
| 52 | News | Noozra | Yes | https://noozra.com/api | Free news headlines from 200+ curated RSS sources |
| 53 | News | Spaceflight News | Yes | https://spaceflightnewsapi.net | Spaceflight related news 🚀 |
| 54 | Open Data | Agent Data Hub | Yes | https://api.agentdatahub.de/docs | Official statistics, central bank rates and FX, with source and licence in every answer |
| 55 | Sports & Fitness | balldontlie | Yes | https://www.balldontlie.io | Balldontlie provides access to stats data from the NBA |
| 56 | Test Data | DummyJSON | Yes | https://dummyjson.com/ | Fake REST API with products, users, posts, comments, todos and more |
| 57 | Test Data | FakerAPI | Yes | https://fakerapi.it/en | APIs collection to get fake data |
| 58 | URL Shorteners | CleanURI | Yes | https://cleanuri.com/docs | URL shortener service |
| 59 | URL Shorteners | GoTiny | Yes | https://github.com/robvanbakel/gotiny-api | A lightweight URL shortener, focused on ease-of-use for the developer and end-user |
| 60 | Weather | NASA POWER | Yes | https://power.larc.nasa.gov/docs/ | Global solar and weather data for energy and agriculture |
| 61 | Weather | Open-Meteo Ensemble | Yes | https://open-meteo.com/en/docs/ensemble-api | Weather ensemble forecasts from multiple models |
| 62 | Weather | Pirate Weather | Yes | https://pirateweather.readthedocs.io/en/latest/ | Free weather API with forecast data similar to Dark Sky |
| 63 | Weather | US Weather | Yes | https://www.weather.gov/documentation/services-web-api | US National Weather Service |
| 64 | Dictionaries | Free Dictionary | Unknown | https://dictionaryapi.dev/ | Definitions, phonetics, pronunciations, parts of speech, examples, synonyms |
| 65 | Games & Comics | Open Trivia | Unknown | https://opentdb.com/api_config.php | Trivia Questions |
| 66 | Open Data | Wikipedia | Unknown | https://www.mediawiki.org/wiki/API:Main_page | Mediawiki Encyclopedia |
| 67 | Science & Math | SpaceX | No | https://github.com/r-spacex/SpaceX-API | Company, vehicle, launchpad and launch data |
| 68 | Social | HackerNews | Unknown | https://github.com/HackerNews/API | Social news for CS and entrepreneurship |
| 69 | Animals | Dog Facts | Yes | https://dukengn.github.io/Dog-facts-API/ | Random dog facts |
| 70 | Anime | AnimeFacts | Yes | https://chandan-02.github.io/anime-facts-rest-api/ | Anime Facts (over 100+) |
| 71 | Anime | AnimeNewsNetwork | Yes | https://www.animenewsnetwork.com/encyclopedia/api.php | Anime industry news |
| 72 | Anime | Catboy | Yes | https://catboys.com/api | Neko images, funny GIFs & more |
| 73 | Anti-Malware | ScanMalware | Yes | https://scanmalware.com | Scan URLs in a sandboxed browser and search past scans by domain, IP, ASN, JARM or favicon hash |
| 74 | Anti-Malware | URLhaus | Yes | https://urlhaus-api.abuse.ch/ | Bulk queries and Download Malware Samples |
| 75 | Art & Design | 3DTexel | Yes | https://3dtexel.com/developers/#public-api | Search free CC0 PBR materials, HDRIs, decals and 3D models for games |
| 76 | Blockchain | ClearTrace | Yes | https://cleartracedata.com/docs | Cross-frontend DEX attribution and execution quality data across Ethereum and L2s |
| 77 | Blockchain | Get Started with Web3 | Yes | https://github.com/beihaili/Get-Started-with-Web3/blob/main/docs/api.md | Bilingual Web3 lessons, glossary search and role-based learning paths |
| 78 | Blockchain | HEY Research Lab | Yes | https://heyresearch.xyz/developers | Builder activity for Robinhood Chain tokens and projects: who is shipping, each ship with its source |
| 79 | Books | Bayan | Yes | https://bayan.alwa7i.com/api | Arabic Islamic encyclopedia: hadith with grades and takhrij, Quran with tafsir, narrators |
| 80 | Books | Bhagavad Gita telugu | Yes | https://gita-api.vercel.app | Bhagavad Gita API in telugu and odia languages |
| 81 | Books | Bible-api | Yes | https://bible-api.com/ | Free Bible API with multiple languages |
| 82 | Books | BookRank | Yes | https://openbookrank.com/en/open-data | Book rankings from open reader ratings: books, series, reading orders and authors |
| 83 | Books | Buku Indonesia | Yes | https://github.com/khrlanamm/buku-indonesia#api | Open data for Indonesian books by ISBN, author, and category |
| 84 | Business | KontragentPro | Yes | https://kontragentpro.ru/api/v2/docs | Russian company data from state registers by INN or OGRN: profile, finances, timeline |
| 85 | Business | Legal Sandbox Georgia | Yes | https://legal.ge/api/openapi.json | Find verified legal specialists in Georgia from natural-language queries |
| 86 | Business | Logoapi | Yes | https://logoapi.sh/docs | SVG logos by domain, stock ticker or crypto symbol, plus favicons and search by name. No signup |
| 87 | Calendar | CountdownsHQ | Yes | https://countdownshq.com/developers | Dates and timezone-correct countdowns for 150+ holidays and observances in 18 countries |
| 88 | Cloud Storage & File Sharing | Deed | Yes | https://deed.page/docs | Publish an HTML file or folder to a permanent public URL with one PUT, no account |
| 89 | Cloud Storage & File Sharing | Pantry | Yes | https://getpantry.cloud/ | Free JSON storage for small projects |
| 90 | Cryptocurrency | Alpha (Mossland) | Yes | https://alpha.moss.land/developers | Korean crypto channel stance + RAG Q&A + canonical entity/topic/event store |
| 91 | Cryptocurrency | bilbop | Yes | https://api.bilbop.org | Solana USDC x402 pay-per-call tools for agents: SPL mint info, token brief, text summarize, Piper TTS, brand feedback |
| 92 | Cryptocurrency | Block Lottos | Yes | https://blocklottos.com/openapi.json | On-chain lottery, draw history, jackpot and advertising endpoints |
| 93 | Cryptocurrency | CoinLobster | Yes | https://coinlobster.com/developers | Live executed whale trades across 15 exchanges and on-chain DEX with an unusualness radar |
| 94 | Currency Exchange | Daleelak | Yes | https://getdaleelak.com/en/api | Egyptian bank and dealer exchange rates, gold and silver prices |
| 95 | Currency Exchange | DolarKaç | Yes | https://dolarkac.com/gelistirici-api | Turkish lira exchange rates, gold and crypto prices from the Istanbul free market |
| 96 | Development | 24 Pull Requests | Yes | https://24pullrequests.com/api | Project to promote open source collaboration during December |
| 97 | Development | Agent Nexus | Yes | https://agentnexus.app/llms.txt | Continuously probed registry of APIs, MCP servers and CLIs callable by agents |
| 98 | Development | AgentPay Domain Lookup | Yes | https://agentpay-lookup.agentpay-apis.workers.dev | DNS records, WHOIS/RDAP, IP ownership and domain reports for AI agents, paid per call via x402 |
| 99 | Development | ApicAgent | Yes | https://www.apicagent.com | Extract device details from user-agent string |
| 100 | Development | BountyCheck | Yes | https://bountycheck.vercel.app | Claimability verdicts for GitHub bounty issues; free stats and sample, checks paid per call via x402 |
| 101 | Development | ChartBytes | Yes | https://chartbytes.meridian-digital.pro | Chart image API: turn a URL into a PNG or SVG chart (bar, line, area, scatter, pie, donut, stacked) |
| 102 | Development | CitedBy | Yes | https://citedby.eu/ai-access-api | Which AI crawlers can read a site: robots.txt and server answer for each |
| 103 | Documents & Productivity | AgentPay Doc Tools | Yes | https://agentpay-tools.agentpay-apis.workers.dev | PDF to text, RSS/Atom/JSON feeds to JSON and sitemaps to URL lists, paid per call via x402 |
| 104 | Documents & Productivity | CleverUtils | Yes | https://cleverutils.com/api | Convert files between 200+ formats, free with no API key |
| 105 | Documents & Productivity | DocStruct | Yes | https://docstruct.pages.dev | AI extraction of invoices, receipts, bank statements & contracts into structured JSON/CSV |
| 106 | Email | AGPC Domain Check | Yes | https://guild.tradeuniquecapital.com/api | Check a domain's SPF, DKIM, DMARC and MX with a graded shareable report |
| 107 | Email | Disify | Yes | https://www.disify.com/ | Validate and detect disposable and temporary email addresses |
| 108 | Email | Email Spam Tester | Yes | https://email-spam-tester.com/api-docs/ | Send a test message and get a spam score, 41 checks with RFC citations and a fix plan |
| 109 | Entertainment | Bucketlist Dream of the Day | Yes | https://bucketlist.nl/samenwerken?lang=en#droom-van-de-dag | Daily travel and life-experience inspiration with canonical links and images |
| 110 | Entertainment | Corporate Buzz Words | Yes | https://github.com/sameerkumar18/corporate-bs-generator-api | REST API for Corporate Buzz Words |
| 111 | Entertainment | CosmyDay Astrology | Yes | https://cosmyday.com/api-docs | Natal charts and sky events computed from Swiss Ephemeris |
| 112 | Entertainment | justmeme.wtf | Yes | https://justmeme.wtf/api-docs | Free meme API with 2400+ templates, search, trending, and AI generation |
| 113 | Environment | Calendrier Tempo | Yes | https://www.calendrier-tempo.fr/api-tempo | French EDF Tempo tariff: today's and tomorrow's colour, kWh prices and 15-day colour forecasts |
| 114 | Events | NeedATicket | Yes | https://needaticket.co.uk/developers | UK and US resale ticket prices from StubHub |
| 115 | Events | Things To Do | Yes | https://thingstodoapi.com/docs/ | Local events in plain words: concerts, pop-ups, markets, classes, gallery nights |
| 116 | Finance | Banking Access Index | Yes | https://www.globalsolo.global/data/banking-access-index | Which US business banking providers accept founders by country of residence, with sources |
| 117 | Finance | Billionaire Tracker | Yes | https://billionairetracker.us/developers | Live net worth, rank and daily change for 3,400+ billionaires, free tier for the top 25 |
| 118 | Finance | bluerate | Yes | https://www.bluerate.ai/developers | Licensed US mortgage loan officers by state, city, ZIP, loan type or language, with NMLS IDs |
| 119 | Finance | Brainy Prices | Yes | https://prices.brainy.ae/developers.html | UAE living costs: KHDA fees, DLD rents, fuel, utilities, telecom and relocation, with dated sources |
| 120 | Finance | contix | Yes | https://contix.es/api/#herramientas | Spain: IBAN validation with bank and BIC, EU VAT number check (VIES), Modelo 303 VAT sums |