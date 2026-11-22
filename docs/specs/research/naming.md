# Research — Product Name Study (OQ-001)

> **Status:** Decided (provisional): **Addisco**, see D-019. Round 3 (developer brief, §15) recommended keeping it; **round 4 ([naming-round-4.md](naming-round-4.md)) re-evaluates from nine angles and recommends Scrubjay or Muista** · v5 · **Last updated:** 2026-11-22
> **Related:** [open-questions OQ-001](../open-questions.md) · [06-cli](../06-cli.md) · [decisions](../decisions.md)

v2 replaced the first 8-name shortlist with a 97-name study (mostly Latin). v3 adds **round 2: 56 names from Ancient Greek, Arabic and other languages** (§10–§14), checked the same way, so 153 candidates in total.

**v4 adds round 3 (§15)** after the pivot to developers (D-027). 74 names in two batches, including the incumbents, were checked against a developer brief: registries, Homebrew and Debian, commands, GitHub, and DNS for `.dev`, `.sh`, `.io` and `.app`. Result: Edisco 4.33, Addisco 4.10, Lampas 4.10, Memini 4.07. Most developer-style names are crowded. The free ones (Memheap, Mempin) read as low-level memory tools. **Recommendation: keep Addisco** (§15.5): the cleanest search footprint, four likely-free domains, and no switching cost.

## Current shortlist (product owner favourites + strongest new contenders)

| Name | Language | Command / alias | Meaning | Score | Main strength | Main risk |
|---|---|---|---|---|---|---|
| **Addisco** ★ | Latin | `addisco` / `addi` | "I keep learning, I add to what I know" | 4.28 | Fits the whole student → lifelong-academic path; free everywhere | Unfamiliar word; "add + disco" reading |
| **Edisco** ★ | Latin | `edisco` / `edi` | "I learn by heart" | 4.10 | Describes spaced repetition exactly; 6 letters | "e-discovery" reading |
| **Lampas** | Greek | `lampas` / `lam` | λαμπάς, the torch passed on in the ancient torch relay | 4.42 | Highest round-2 score; easy to say and spell; free everywhere; knowledge handed on through generations | Also a fabric and an equine mouth condition; less directly about *study* |
| **Prokope** | Greek | `prokope` / `prok` | προκοπή, the Stoic word for steady progress toward mastery | 4.10 | Captures autonomy + mastery + progress tracking | Can be read "pro-cope" |
| **Spoude** | Greek | `spoude` / `spo` | σπουδή, zeal, earnest study | 4.30 | Very scarce (10 repos); 6 letters | Hard to spell from sound (SPOO-day) |
| **Melete** | Greek | `melete` / `mel` | μελέτη, study/practice; the Muse of practice | 3.83 | Best story in round 2 (one of the three original Muses) | Crate taken; a "Meleti" study app is on a waitlist |
| **Dirasa** | Arabic | `dirasa` / `dira` | دِرَاسَة, "study" | 4.17 | Literally the right word; free everywhere | The everyday Arabic word, so it's generic in Arabic-speaking markets |
| **Tahsil** | Arabic | `tahsil` / `tah` | تَحْصِيل, attainment of knowledge | 3.75 | Scholarly, secular | "Tehsil" = South Asian administrative district |

★ = product-owner favourite. Full reasoning: Latin round §4, Greek/Arabic round §11.

## 1. Brief

- **Meaning:** study, mastery, autonomy, scholarship. The name must age well along the audience path: university student → postgraduate → lifelong academic (D-018). It should be serious but warm, never childish.
- **CLI:** users type it in a terminal many times a day, so the full command must be short and unambiguous, with a short alias.
- **Uniqueness / scarcity:** no existing study or education product with the same or a near-identical name; package names free; low noise on GitHub.

## 2. Method

1. **Candidate pool (97 names)** from eight sources: Latin study and learning verbs, Latin compound learning verbs (*e-*, *ad-*, *con-*, *per-*, *prae-disco*), Greek scholarship terms, mastery and discipline words, autonomy words, academic objects (carrel, lectern…), non-English learning words, and coined forms.
2. **Automated checks** for every name, run 2026-11-19 from this repo's environment:
   - Bare-name availability on **crates.io** (matters for `cargo install`), **npm** and **PyPI**. npm can fall back to a scope (`@name/cli`), so it's weighted lower.
   - **GitHub** repositories with the word in their name (search API, `in:name`). This is a rough noise measure: prefix matches inflate it (e.g. *edisco* matches *ediscovery*, *mathema* matches *mathematics*).
   - **Command collisions:** the name and its alias are compared against ~1,860 commands. That's every command in this Linux container plus a curated list of popular CLIs, PowerShell default aliases, `cmd.exe` built-ins and macOS tools.
   - **Typing ergonomics:** length and left/right-hand alternation on QWERTY. Alternating hands is faster to type.
3. **Web checks** for existing apps, products and companies: 30 searches covering the 45 most promising names. Other conflicts come from general knowledge and are labelled "(knowledge)".
4. **Not checked** (blocked from this environment or out of scope): domain registration, Homebrew/Debian package names, and **trademark registers**. See §7.

## 3. Scoring

Each criterion is scored 1–5. The weights reflect the brief: uniqueness and CLI fit first.

| Criterion | Weight | How it's scored |
|---|---|---|
| **Uniqueness (Uniq)** | 35% | 5, minus: crates taken −1, npm −0.5, PyPI −0.5; GitHub ≥25 repos −0.5, ≥100 −1, ≥500 −1.5, ≥2000 −2; web conflict H −2.5, M −1, unchecked −0.5 (floor 1) |
| **CLI fit (CLI)** | 25% | Length: ≤6 letters = 5, 7 = 4, 8 = 3, 9 = 2, ≥10 = 1; ±0.5 for good/poor hand alternation |
| **Meaning (Mean)** | 25% | Fit with study / mastery / autonomy / scholarship, and how well it ages from undergrad to professor (judgement) |
| **Say & spell (Say)** | 15% | Can someone type it correctly after hearing it once? (judgement) |

**Known-conflict levels:** **H** = an existing study/education/productivity product with the same or a near-identical name, or a dominant brand. **M** = a related or near-name product in a nearby field, or a generic word. **L** = nothing relevant found. **-** = not web-checked.

## 4. Top 10

| # | Name | Command / alias | Story | Strengths | Risks |
|---|---|---|---|---|---|
| 1 | **Addisco** | `addisco` / `addi` | Latin *addisco*: "I learn in addition, I keep learning". Learning that never stops. | Fits the whole audience path (student → lifelong academic). All registries free; 39 GitHub repos; no products found. 7 letters, easy to type. | Unfamiliar word. Stress varies (ad-DIS-ko). Read as "add + disco" by some. |
| 2 | **Lucubra** | `lucubra` / `luc` | From *lucubrare*, "to study by lamplight", the root of English *lucubration* | Highest score. Strongest imagery (a lamp, a night scholar). All registries free; only 19 GitHub repos; nothing found. | Hard to spell from hearing (LOO-kyoo-bruh). Possible unflattering associations in English. |
| 3 | **Edisco** | `edisco` / `edi` | Latin *edisco*: "I learn thoroughly, I learn by heart" | Describes spaced repetition exactly. 6 letters. All registries free; no products found. | Can read as "e-discovery" (legal tech), which inflates the GitHub count (234). |
| 4 | **Peritia** | `peritia` / `peri` | Latin *peritia*: expertise gained through experience | Mastery theme. All registries free; nothing found; 7 letters. | Pronunciation (peh-RIT-ee-uh). Less about *study* itself. |
| 5 | **Sciora** | `sciora` / `sci` | Coined from Latin *scio*, "I know" | Very scarce (9 GitHub repos, nothing found). 6 letters, `sci` is a natural alias. | Coined word, so the meaning has to be explained. Could be misheard as "Sierra". |
| 6 | **Assidua** | `assidua` / `asd` | Latin *assidua*: persistent, diligent | Scarce (6 repos), all free. | Weaker brand feel. `asd` is a keyboard-mash string. |
| 7 | **Sedula** | `sedula` / `sdu` | Latin *sedula*: diligent | Scarce, all free, 6 letters. | Weak meaning fit, flat sound. |
| 8 | **Docet** | `docet` / `doc` | *Experientia docet*: "experience teaches" | 5 letters, all free. | `doc` alias is generic. It's about teaching rather than learning. |
| 9 | **Condisco** | `condisco` / `cds` | Latin "I learn thoroughly" | Only 1 GitHub repo. | Reads as "con disco"; 8 letters. |
| 10 | **Discimus** | `discimus` / `disc` | *Docendo discimus*: "by teaching, we learn" | Community/teaching story that suits PER-6 and PER-8 later. Scarce. | 8 letters; pronunciation (DIS-ki-moos). |

**Worth considering despite a lower score:**
- **Studium** (#12): the best meaning and the easiest to say. But there's an existing *StudiumApp* (school communications, India), and in German "Studium" simply means "university studies", so it's generic and unsearchable in German-speaking markets.
- **Carrel** (#30): a perfect academic image (the library study desk that postgrads get). No products found, but as a dictionary word every package name is taken.

## 5. Notable eliminations

| Name | Why it's out |
|---|---|
| Scholar | Google Scholar dominates the term in our exact audience |
| Sapere | "Sapere: Learn in Minutes" iOS learning app (2025) |
| Scholaris | "Scholaris" school-management platform; 9 letters |
| Studeo | Funded tutoring app with spaced repetition and flashcards |
| Schole | "Scholé" AI learning platform |
| Mathesis | Offline spaced-repetition learning app |
| Doctus | Microlearning app |
| Skolar | "Skolar: AI Flashcards & Notes" |
| Erudy | "Erudity" flashcard app |
| Studix | AI study platform |
| Akademos | "Akademus AI" study app |
| Askesis | Habit-tracker app + coaching SaaS |
| Mnemosyne | Long-standing spaced-repetition app |
| Studia | GCSE revision planner + "Studia – Quiz & Flashcard AI" |
| Mastra, Eruda | Popular developer tools (AI framework; web console) |
| Magister, Lectio | National school-management systems (Netherlands, Denmark) |
| Bildung | Generic German word; also the App Store's German "Education" category label |

## 6. CLI naming conventions (applies to whichever name is chosen)

- **Binary = full name**, lowercase (e.g. `addisco`). It's always available and safe in scripts and docs.
- **One official short alias** = the first 3–4 letters (e.g. `addi`). Easy to remember, and it matches what users type before pressing Tab. It's installed only if nothing with that name is already on `PATH` (CLI-018).
- Built-in short subcommands make daily use quick: `addi r` (review), `addi s` (session), `addi t 25m` (timer). These are user-editable in `cli.toml` ([06 §7](../06-cli.md#7-configuration-clitoml)).
- Use the same name for the binary, crate, npm scope, Homebrew formula, winget id and URL scheme (`addisco://`).

## 7. Before committing to a name

- [ ] Trademark search in classes 9, 41, 42: USPTO (US), EUIPO (EU), UKIPO (UK), WIPO Global Brand Database
- [ ] Domains: `.com`, `.app`, `.io` (couldn't be checked from this environment)
- [ ] App Store / Google Play name search; GitHub org; social handles
- [ ] Say it aloud to 5–10 target users (undergrad, postgrad, academic): can they spell it after hearing it? Any unwanted associations in their languages?
- [ ] Reserve the crates.io name, npm scope and GitHub org on the day of decision
- [ ] Resolve OQ-001; replace the working titles (`StudyApp`, `study`, `studyapp://`) across the repo

## 8. Full ranked list (97 candidates)

Registry columns: ✓ = bare name free, ✗ = taken (snapshot 2026-11-19). GitHub = repositories with the word in their name (rough; prefix matches inflate it).

| # | Name | Origin — meaning | Len | Alias | crates · npm · PyPI | GitHub repos | Known conflicts | Uniq | CLI | Mean | Say | **Score** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | **Lucubra** | Latin — from lucubrare: to study late by lamplight | 7 | `luc` | ✓ ✓ ✓ | 19 | L: No app found | 5 | 4 | 5 | 2 | **4.30** |
| 2 | **Addisco** | Latin — I learn further, in addition — learning that keeps going | 7 | `addi` | ✓ ✓ ✓ | 39 | L: No app or company found | 4.5 | 4 | 5 | 3 | **4.28** |
| 3 | **Sciora** | Coined — from Latin scio (I know) | 6 | `sci` | ✓ ✓ ✓ | 9 | L: No app found | 5 | 5 | 3 | 3 | **4.20** |
| 4 | **Peritia** | Latin — expertise, skill from experience | 7 | `peri` | ✓ ✓ ✓ | 48 | L: No app found | 4.5 | 4.5 | 4 | 3 | **4.15** |
| 5 | **Edisco** | Latin — I learn thoroughly / by heart (ediscere) | 6 | `edi` | ✓ ✓ ✓ | 234 | L: No app or company found | 4 | 5 | 4 | 3 | **4.10** |
| 6 | **Assidua** | Latin — assiduous, persistent | 7 | `asd` | ✓ ✓ ✓ | 6 | L: No app found | 5 | 4.5 | 3 | 3 | **4.08** |
| 7 | **Sedula** | Latin — diligent, sedulous | 6 | `sdu` | ✓ ✓ ✓ | 8 | L: not web-checked | 5 | 5 | 2 | 3 | **3.95** |
| 8 | **Docet** | Latin — it teaches — experientia docet | 5 | `doc` | ✓ ✓ ✓ | 105 | L: No app found | 4 | 5 | 3 | 3 | **3.85** |
| 9 | **Condisco** | Latin — I learn thoroughly | 8 | `cds` | ✓ ✓ ✓ | 1 | L: Not found; reads as "con disco" | 5 | 3.5 | 3 | 3 | **3.83** |
| 10 | **Discimus** | Latin — we learn — from "docendo discimus" (by teaching we learn) | 8 | `disc` | ✓ ✓ ✓ | 48 | L: No app found | 4.5 | 3 | 4 | 3 | **3.78** |
| 11 | **Docendo** | Latin — by teaching — docendo discimus | 7 | `dcd` | ✓ ✓ ✓ | 12 | M: Docendo: Spanish school-management app | 4 | 4.5 | 3 | 3 | **3.72** |
| 12 | **Studium** | Latin — study, zeal; medieval word for a university (studium generale) | 7 | `stu` | ✓ ✗ ✓ | 1,491 | M: StudiumApp (Indian school-comms app); generic German word for "university studies" | 2.0 | 4 | 5 | 5 | **3.70** |
| 13 | **Shugyo** | Japanese — austere training toward mastery (修行) | 6 | `shg` | ✓ ✓ ✓ | 48 | -: not web-checked | 4.0 | 5 | 3 | 2 | **3.70** |
| 14 | **Discio** | Coined — from discere | 6 | `dsi` | ✓ ✗ ✓ | 14 | M: npm taken | 3.5 | 5 | 3 | 3 | **3.67** |
| 15 | **Gakumon** | Japanese — learning, scholarship (学問) | 7 | `gkm` | ✓ ✓ ✓ | 22 | -: not web-checked | 4.5 | 3.5 | 3 | 3 | **3.65** |
| 16 | **Lerno** | Coined — from Esperanto lerni (to learn) | 5 | `lrn` | ✓ ✓ ✓ | 283 | M: Lernu school planner, "Lernio" (near) | 3 | 5 | 3 | 4 | **3.65** |
| 17 | **Mathetes** | Greek — learner, disciple | 8 | `mtt` | ✓ ✓ ✓ | 65 | L: Mathetis Bible-study app (near) | 4.5 | 3 | 4 | 2 | **3.62** |
| 18 | **Glaux** | Greek — Athena's owl, symbol of wisdom | 5 | `glx` | ✗ ✓ ✗ | 55 | L: No app found; crate and PyPI taken | 3.0 | 5 | 4 | 2 | **3.60** |
| 19 | **Perdisco** | Latin — I learn completely (perdiscere) | 8 | `pds` | ✓ ✓ ✓ | 10 | M: Not found in search, but recalled as an accounting-education publisher; verify | 4 | 3 | 4 | 3 | **3.60** |
| 20 | **Disce** | Latin — Learn! (imperative; motto "disce aut discede") | 5 | `dsc` | ✓ ✓ ✓ | 970 | M: Discite Latin and learning-path apps; Disco learning platform | 2.5 | 5 | 4 | 3 | **3.58** |
| 21 | **Lucerna** | Latin — oil lamp (of the night scholar) | 7 | `lcn` | ✓ ✓ ✗ | 206 | L: Defunct kids-learning studio (2018) | 3.5 | 4 | 3 | 4 | **3.57** |
| 22 | **Erudio** | Latin — I educate (lit. bring out of roughness) | 6 | `eru` | ✓ ✓ ✓ | 2,562 | M: Erudio: Cambridge Latin Course app; consultancies | 2 | 5 | 4 | 4 | **3.55** |
| 23 | **Discit** | Latin — (one) learns — from "qui docet discit" | 6 | `dst` | ✓ ✓ ✓ | 110 | M: Discite apps (near-identical) | 3 | 5 | 3 | 3 | **3.50** |
| 24 | **Peripatos** | Greek — walkway where Aristotle taught | 9 | `ppt` | ✓ ✓ ✓ | 22 | L: No app found | 5 | 2.5 | 3 | 2 | **3.42** |
| 25 | **Studeo** | Latin — I study | 6 | `sto` | ✓ ✓ ✓ | 174 | H: Studeo: funded tutoring app with spaced repetition | 1.5 | 5 | 4 | 4 | **3.38** |
| 26 | **Doctus** | Latin — learned, scholarly | 6 | `dtu` | ✓ ✓ ✓ | 130 | H: Doctus: microlearning app | 1.5 | 5 | 4 | 4 | **3.38** |
| 27 | **Studia** | Latin — studies (plural of studium) | 6 | `sda` | ✓ ✗ ✓ | 3,234 | H: Studia revision planner and "Studia – Quiz & Flashcard AI" apps | 1 | 5 | 4 | 5 | **3.35** |
| 28 | **Scholarch** | Greek — head of an ancient school of philosophy | 9 | `sca` | ✓ ✓ ✓ | 124 | L: No app found | 4 | 2 | 4 | 3 | **3.35** |
| 29 | **Habitus** | Latin — ingrained disposition/habit | 7 | `hbt` | ✓ ✗ ✓ | 431 | M: not web-checked; common term | 2.5 | 4.5 | 3 | 4 | **3.35** |
| 30 | **Carrel** | English — private library study desk | 6 | `car` | ✗ ✗ ✗ | 514 | L: No app found, but every registry name taken (dictionary word) | 1.5 | 4.5 | 5 | 3 | **3.35** |
| 31 | **Scholr** | Coined — scholar, vowel-dropped | 6 | `shr` | ✓ ✓ ✗ | 130 | M: PyPI taken; misspelling risk | 2.5 | 5 | 3 | 3 | **3.33** |
| 32 | **Erudy** | Coined — from erudite + study | 5 | `erd` | ✓ ✓ ✓ | 13 | H: Erudity flashcard app (near-identical) | 2.5 | 5 | 3 | 3 | **3.33** |
| 33 | **Sapere** | Latin — to know — from "sapere aude": dare to know | 6 | `spr` | ✓ ✗ ✗ | 179 | H: Sapere: Learn in Minutes iOS learning app (2025) | 1 | 5 | 5 | 3 | **3.30** |
| 34 | **Studix** | Coined — study + -ix | 6 | `stx` | ✓ ✓ ✓ | 78 | H: Studix: AI study platform | 2.0 | 5 | 3 | 4 | **3.30** |
| 35 | **Praedisco** | Latin — I learn beforehand (exam prep) | 9 | `prd` | ✓ ✓ ✓ | 0 | L: Not found | 5 | 2 | 3 | 2 | **3.30** |
| 36 | **Studiolo** | Italian — Renaissance scholar's private study room | 8 | `sdl` | ✗ ✓ ✓ | 124 | L: No app found; crate taken | 3 | 3 | 4 | 3 | **3.25** |
| 37 | **Scholaris** | Latin — of scholars | 9 | `sch` | ✓ ✓ ✗ | 182 | M: Scholaris school-management platform | 2.5 | 2 | 5 | 4 | **3.23** |
| 38 | **Aude** | Latin — dare — from "sapere aude" | 4 | `aud` | ✗ ✓ ✓ | 1,758 | M: not web-checked; French department name | 1.5 | 5 | 4 | 3 | **3.22** |
| 39 | **Arete** | Greek — excellence, mastery | 5 | `art` | ✗ ✗ ✗ | 1,019 | H: Crowded (knowledge) | 1 | 4.5 | 5 | 3 | **3.17** |
| 40 | **Schole** | Greek — leisure devoted to learning; root of school/scholar | 6 | `skl` | ✓ ✓ ✓ | 3,624 | H: Scholé: AI learning platform | 1 | 5 | 5 | 2 | **3.15** |
| 41 | **Quaero** | Latin — I seek, I inquire (research) | 6 | `qro` | ✗ ✓ ✗ | 84 | M: Crate and PyPI taken; Quaero research project | 2.0 | 5 | 3 | 3 | **3.15** |
| 42 | **Discere** | Latin — to learn | 7 | `dcr` | ✓ ✓ ✗ | 109 | -: not web-checked | 3.0 | 4 | 3 | 2 | **3.10** |
| 43 | **Lector** | Latin — reader; university lecturer | 6 | `lct` | ✗ ✗ ✗ | 5,562 | H: Common app name (knowledge) | 1 | 5 | 3 | 5 | **3.10** |
| 44 | **Quaestio** | Latin — scholastic question for inquiry | 8 | `qst` | ✓ ✓ ✓ | 379 | M: Questio flashcard app; "Quaestio" card game | 3 | 3 | 4 | 2 | **3.10** |
| 45 | **Askesis** | Greek — disciplined practice toward mastery | 7 | `aks` | ✓ ✓ ✗ | 55 | H: Askesis: habit-tracker app and coaching SaaS | 1.5 | 4.5 | 4 | 3 | **3.10** |
| 46 | **Bildung** | German — self-formation through education (Humboldt) | 7 | `bld` | ✓ ✓ ✓ | 355 | H: Generic German word; App Store's German "Education" category label | 1.5 | 4.5 | 4 | 3 | **3.10** |
| 47 | **Lumen** | Latin — light | 5 | `lum` | ✗ ✗ ✗ | 23,849 | H: Lumen Learning (edtech) (knowledge) | 1 | 5 | 3 | 5 | **3.10** |
| 48 | **Lyceum** | Greek — Aristotle's school | 6 | `lyc` | ✓ ✗ ✗ | 2,130 | H: Crowded (knowledge) | 1 | 5 | 4 | 3 | **3.05** |
| 49 | **Kenning** | Old Norse — ken = range of knowledge | 7 | `kn` | ✓ ✗ ✗ | 121 | M: not web-checked | 2.0 | 4 | 3 | 4 | **3.05** |
| 50 | **Eruditus** | Latin — learned, erudite | 8 | `edt` | ✓ ✓ ✓ | 90 | H: Eruditus: large edtech company | 2.0 | 3.5 | 4 | 3 | **3.03** |
| 51 | **Skolar** | Coined — scholar respelled | 6 | `skr` | ✓ ✓ ✓ | 439 | H: Skolar: AI Flashcards & Notes app; Brazilian school app | 1.5 | 5 | 3 | 3 | **2.97** |
| 52 | **Sylla** | Coined — from syllabus | 5 | `syl` | ✓ ✗ ✓ | 11,207 | M: not web-checked | 1.5 | 5 | 3 | 3 | **2.97** |
| 53 | **Studiosus** | Latin — eager student; a scholar | 9 | `sts` | ✓ ✓ ✓ | 5 | H: Studiosus: major German study-tour operator | 2.5 | 2.5 | 4 | 3 | **2.95** |
| 54 | **Lectio** | Latin — a reading; lecture | 6 | `lec` | ✗ ✗ ✗ | 4,427 | H: Lectio: Danish schools' admin platform (knowledge) | 1 | 5 | 3 | 4 | **2.95** |
| 55 | **Tirocinium** | Latin — apprenticeship, first training | 10 | `tir` | ✓ ✓ ✓ | 7 | L: not web-checked | 5 | 1 | 3 | 1 | **2.90** |
| 56 | **Akademos** | Greek — grove of Plato's Academy | 8 | `akd` | ✓ ✓ ✓ | 36 | H: Akademus AI study app (near-identical) | 2.0 | 3 | 4 | 3 | **2.90** |
| 57 | **Mathema** | Greek — that which is learned | 7 | `mtm` | ✓ ✗ ✗ | 33,627 | -: not web-checked | 1.5 | 4.5 | 3 | 3 | **2.85** |
| 58 | **Lectern** | English — reading stand | 7 | `lcr` | ✗ ✗ ✗ | 363 | M: not web-checked | 1 | 4 | 3 | 5 | **2.85** |
| 59 | **Magister** | Latin — master, teacher | 8 | `mag` | ✓ ✗ ✓ | 2,821 | H: Magister: Dutch school-management system (knowledge) | 1 | 3 | 4 | 5 | **2.85** |
| 60 | **Mastra** | Coined — from mastery | 6 | `mst` | ✓ ✗ ✓ | 3,065 | H: Mastra: popular TypeScript AI framework (knowledge) | 1 | 4.5 | 3 | 4 | **2.83** |
| 61 | **Scholia** | Greek — scholars' marginal commentaries | 7 | `sco` | ✗ ✗ ✗ | 173 | H: Scholia: Wikidata scholarly-profiles tool (knowledge) | 1 | 4 | 4 | 3 | **2.80** |
| 62 | **Noesis** | Greek — understanding, intellection | 6 | `nsi` | ✗ ✗ ✗ | 1,070 | H: Common (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 63 | **Techne** | Greek — craft knowledge, art | 6 | `tch` | ✗ ✗ ✗ | 5,531 | H: Crowded (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 64 | **Noctua** | Latin — little owl (Athena's) | 6 | `nct` | ✗ ✗ ✗ | 501 | H: Noctua: PC-cooling brand (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 65 | **Metis** | Greek — wisdom, practical intelligence | 5 | `mts` | ✗ ✗ ✗ | 3,656 | H: Crowded (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 66 | **Vidya** | Sanskrit — knowledge, learning | 5 | `vdy` | ✗ ✗ ✗ | 12,239 | H: Crowded (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 67 | **Eruda** | Coined — from erudite | 5 | `erd2` | ✓ ✗ ✓ | 170 | H: Eruda: popular web dev-tools console (knowledge) | 1 | 5 | 3 | 3 | **2.80** |
| 68 | **Paideia** | Greek — education and self-cultivation | 7 | `pdi` | ✓ ✓ ✗ | 332 | H: Paideia Institute and others (knowledge) | 1 | 4.5 | 4 | 2 | **2.77** |
| 69 | **Mneme** | Greek — memory; muse of memory | 5 | `mne` | ✗ ✗ ✗ | 463 | H: Crowded; hard to spell | 1 | 5 | 4 | 1 | **2.75** |
| 70 | **Didasko** | Greek — I teach | 7 | `dds` | ✓ ✓ ✓ | 45 | H: Didasko: teacher course-handbook app | 2.0 | 4 | 3 | 2 | **2.75** |
| 71 | **Polymath** | Greek — person of wide learning | 8 | `pmt` | ✗ ✗ ✗ | 1,443 | H: Crowded (knowledge) | 1 | 2.5 | 4 | 5 | **2.73** |
| 72 | **Folio** | English — large book page/format | 5 | `fol` | ✗ ✗ ✗ | 55,943 | H: Crowded (knowledge) | 1 | 4.5 | 2 | 5 | **2.73** |
| 73 | **Glossa** | Latin — marginal explanatory note | 6 | `gls` | ✗ ✗ ✗ | 7,773 | H: Common (knowledge) | 1 | 5 | 2 | 4 | **2.70** |
| 74 | **Organon** | Greek — instrument of thought (Aristotle) | 7 | `org` | ✗ ✗ ✗ | 166 | H: Organon & Co. (pharma) (knowledge) | 1 | 4 | 3 | 4 | **2.70** |
| 75 | **Ingenium** | Latin — intellect, talent, ingenuity | 8 | `ing` | ✓ ✗ ✓ | 649 | M: Common company name (knowledge) | 2.0 | 2.5 | 3 | 4 | **2.68** |
| 76 | **Autodidact** | Greek — self-taught person | 10 | `adt` | ✓ ✗ ✗ | 480 | M: Crowded concept; npm/PyPI taken | 2.0 | 1.5 | 4 | 4 | **2.68** |
| 77 | **Mathesis** | Greek — learning, the act of learning | 8 | `mth` | ✓ ✓ ✗ | 278 | H: Mathesis: offline spaced-repetition learning app | 1 | 3.5 | 4 | 3 | **2.67** |
| 78 | **Ilm** | Arabic — knowledge | 3 | `ilm` | ✗ ✗ ✓ | 8,653 | H: Crowded (knowledge) | 1 | 4.5 | 3 | 3 | **2.67** |
| 79 | **Hikma** | Arabic — wisdom | 5 | `hkm` | ✓ ✗ ✓ | 2,221 | H: Crowded (knowledge) | 1 | 4.5 | 3 | 3 | **2.67** |
| 80 | **Sadhana** | Sanskrit — disciplined daily practice | 7 | `sdh` | ✓ ✓ ✓ | 2,228 | H: Many apps (knowledge) | 1 | 4.5 | 3 | 3 | **2.67** |
| 81 | **Sententia** | Latin — opinion, maxim; scholastic "Sentences" | 9 | `sen` | ✓ ✓ ✗ | 121 | -: not web-checked | 3.0 | 2.5 | 2 | 3 | **2.62** |
| 82 | **Vellum** | English — manuscript parchment | 6 | `vlm` | ✗ ✗ ✗ | 1,279 | H: Vellum: book-formatting app (knowledge) | 1 | 4.5 | 2 | 4 | **2.58** |
| 83 | **Autark** | Greek — self-sufficient | 6 | `atk` | ✗ ✗ ✗ | 156 | M: not web-checked | 1 | 5 | 2 | 3 | **2.55** |
| 84 | **Quire** | English — gathering of manuscript pages | 5 | `qr` | ✗ ✗ ✗ | 881 | H: Quire: task-management app (knowledge) | 1 | 5 | 2 | 3 | **2.55** |
| 85 | **Florilegium** | Latin — anthology of gathered excerpts | 11 | `flg` | ✗ ✓ ✓ | 73 | L: not web-checked | 3.5 | 1 | 3 | 1 | **2.37** |
| 86 | **Episteme** | Greek — scientific knowledge | 8 | `eps` | ✗ ✗ ✗ | 295 | H: Common (knowledge) | 1 | 3 | 3 | 3 | **2.30** |
| 87 | **Disputatio** | Latin — scholastic debate | 10 | `dsp` | ✓ ✗ ✓ | 88 | -: not web-checked | 3.5 | 1 | 2 | 2 | **2.27** |
| 88 | **Athenaeum** | Greek — temple of Athena; learned institution | 9 | `ath` | ✓ ✗ ✗ | 490 | H: Crowded (knowledge) | 1 | 2.5 | 4 | 2 | **2.27** |
| 89 | **Vademecum** | Latin — go with me: a handbook carried everywhere | 9 | `vdm` | ✗ ✓ ✓ | 255 | M: Generic term for medical/handbook apps (knowledge) | 2 | 2 | 3 | 2 | **2.25** |
| 90 | **Xuexi** | Chinese — to study (学习) | 5 | `xx` | ✗ ✗ ✓ | 4,106 | H: Xuexi Qiangguo: Chinese state study app (knowledge) | 1 | 5 | 2 | 1 | **2.25** |
| 91 | **Enchiridion** | Greek — handbook (Epictetus) | 11 | `ench` | ✓ ✗ ✓ | 140 | M: not web-checked | 2.5 | 1.5 | 3 | 1 | **2.15** |
| 92 | **Mnemosyne** | Greek — goddess of memory | 9 | `mns` | ✗ ✗ ✗ | 1,309 | H: Mnemosyne: established spaced-repetition app (knowledge) | 1 | 2.5 | 4 | 1 | **2.12** |
| 93 | **Commonplace** | English — commonplace book of collected knowledge | 11 | `cpl` | ✗ ✗ ✗ | 563 | H: Crowded (knowledge) | 1 | 1 | 3 | 5 | **2.10** |
| 94 | **Anamnesis** | Greek — Platonic recollection | 9 | `anm` | ✗ ✗ ✗ | 413 | M: Medical term | 1 | 2.5 | 3 | 2 | **2.02** |
| 95 | **Colophon** | English — end-note of a manuscript | 8 | `cph` | ✗ ✗ ✗ | 210 | M: not web-checked | 1 | 2.5 | 2 | 3 | **1.93** |
| 96 | **Scriptorium** | Latin — monastery writing room | 11 | `scr` | ✗ ✗ ✗ | 870 | H: Crowded (knowledge) | 1 | 1 | 3 | 3 | **1.80** |
| 97 | **Marginalia** | English — notes in the margin | 10 | `mgl` | ✗ ✗ ✗ | 1,108 | H: Crowded (knowledge) | 1 | 1 | 3 | 3 | **1.80** |

## 9. Sources for web conflict checks

- StudiumApp: <https://apps.apple.com/at/app/studiumapp/id1637890861?l=en-GB>
- Studeo: <https://apps.apple.com/us/app/studeo/id1564567385>
- Sapere: Learn in Minutes: <https://apps.apple.com/us/app/-/id6746458440>
- Scholaris: <https://scholariss.netlify.app/>
- Scholé: <https://www.producthunt.com/posts/schole-learn-by-building>
- Mathesis: <https://apps.apple.com/us/app/mathesis/id6757658558>
- Doctus: Microlearning: <https://mwm.ai/apps/doctus-microlearning/6756780930>
- Skolar: AI Flashcards & Notes: <https://mwm.ai/apps/skolar-ai-flashcards-notes/6751448362>
- Erudity: <https://www.producthunt.com/products/erudity>
- Studix: <https://webcatalog.io/en/apps/studixzone>
- Akademus AI: <https://apps.apple.com/app/id6748606960>
- Askesis: Daily Habit Tracker: <https://apps.apple.com/us/app/-/id6755964020>
- Studia – Quiz & Flashcard AI: <https://apps.apple.com/vc/app/studia-quiz-flashcard-ai/id6741497812>
- Discite: <https://apps.apple.com/us/app/discite/id6471923551>
- Docendo: <https://apps.apple.com/si/app/docendo/id1139783020>
- Erudio: Cambridge Latin Course: <https://apps.apple.com/us/app/id1219540268>
- Didasko: <https://apps.apple.com/lb/app/didasko/id1441181595>
- Questio: <https://apps.apple.com/my/app/questio/id1572090054>

---

# Round 2 — Ancient Greek, Arabic and other languages

## 10. Method (round 2)

Same pipeline and the same scoring weights as round 1 (§2–§3), applied to **56 new names**: 20 Ancient Greek, 25 Arabic, and 11 from other languages for contrast (Japanese, Hebrew, Persian, Sanskrit, Welsh, Finnish, Swahili). Meanings were taken from the original-script words (shown in the table). 25 web searches covered the 40 strongest names.

**Consistency rule:** a name that's the everyday word for "study" in a major language gets the same "generic" penalty Studium got for German (Dirasa, Dars, Benkyo, Danesh, Elimu).

## 11. Round-2 results

| # | Name | Command / alias | Original · meaning | Score | Verdict |
|---|---|---|---|---|---|
| 1 | **Lampas** | `lampas` / `lam` | λαμπάς · torch of the relay race (passing knowledge on) | 4.42 | **Strong contender.** Easy to say and spell, free everywhere, no products found |
| 2 | Shinun | `shinun` / `shin` | שִׁנּוּן · learning by repetition | 4.33 | Good meaning, but Hebrew religious-study term; Japanese speakers may hear "shinu" (to die) |
| 3 | **Spoude** | `spoude` / `spo` | σπουδή · zeal, earnest study | 4.30 | Scarce and short; spelling from sound is the weak point |
| 4 | **Dirasa** | `dirasa` / `dira` | دِرَاسَة · study | 4.17 | The best Arabic option on registries; generic in Arabic markets |
| 5 | Kepos | `kepos` / `kep` | κῆπος · Epicurus's "Garden" school | 4.17 | Lovely "cultivate your own study garden" angle (fits customisation), but less about study |
| 6 | **Prokope** | `prokope` / `prok` | προκοπή · Stoic progress toward mastery | 4.10 | Strongest *meaning* for this product among the Greek names |
| 7 | Tanren | `tanren` / `tan` | 鍛錬 · forging, disciplined training | 4.07 | Good mastery story; npm/PyPI taken |
| 8 | Lychnos | `lychnos` / `lyc` | λύχνος · the scholar's lamp | 4.05 | Greek twin of Lucubra; hard to spell |
| 9 | Manthano | `manthano` / `mant` | μανθάνω · "I learn" | 3.90 | Clear meaning; 8 letters |
| 10 | **Melete** | `melete` / `mel` | μελέτη · study/practice; the Muse of practice | 3.83 | Best story; crate taken and a near-identical study app in pre-launch |

## 12. What round 2 taught us

- **Arabic study vocabulary is heavily occupied by Quran-study apps.** Itqan (mastery), Muraja (revision), Takrar (repetition), Siraj (lamp), Midad (ink) and Marifa (knowledge) are all existing Quran memorisation or Islamic-learning apps. Using one would position the product as a religious study app, and they'd be hard to rank against. The general, secular words (Dirasa, Tahsil) are free, but generic.
- **Religious and political load:** Ijtihad, Ijaza, Halaqa, Talab and Alim carry religious-scholarly meaning that could alienate some users or misrepresent the product. Madrasa, Talib and Majlis were excluded up front for that reason. Ijaza is the best story among them (the licence to teach, a forerunner of the university degree) but needs care.
- **Cross-language false friends are real:** Malaka (Ibn Khaldun's "acquired mastery", a perfect meaning) is vulgar slang in Greek, so it's disqualified. Every finalist needs a check with native speakers of major languages.
- **Transliteration hurts CLI typing.** Arabic names have several valid spellings (Dirasa/Dirasah, Muraja/Muraja'ah, Mudhakara/Muzakra), and Greek ones bring *ch*, *ph*, *ou* (Lychnos, Mouseion). Users have to type the exact spelling, so choose the most common spelling and reserve the variants as redirects.
- **Greek scholarly vocabulary is less occupied than Latin**: the Muses, Stoic terms and school names. That's where the scarce, meaningful options are (Lampas, Prokope, Spoude, Melete).
- **Direct competitors found:** *Hexis* is a local-first desktop app for tasks, goals, habits and timers; *Anodos* is a spaced-retention learning app. Both would have been natural names.

## 13. Full round-2 list (56 candidates)

| # | Name | Language · original | Meaning | Len | Alias | crates · npm · PyPI | GitHub repos | Known conflicts | Uniq | CLI | Mean | Say | **Score** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | **Lampas** | Greek · λαμπάς | torch; the torch relay that passes the flame on | 6 | `lam` | ✓ ✓ ✓ | 87 | L: Nothing found; also a fabric and an equine mouth condition | 4.5 | 5 | 4 | 4 | **4.42** |
| 2 | **Shinun** | Hebrew · שִׁנּוּן | learning by repetition (lit. "sharpening") | 6 | `shin` | ✓ ✓ ✓ | 14 | L: Nothing found; Talmudic term for memorising by repetition. Japanese speakers may hear "shinu" (to die) | 5 | 4.5 | 4 | 3 | **4.33** |
| 3 | **Spoude** | Greek · σπουδή | earnestness, zeal; serious study | 6 | `spo` | ✓ ✓ ✓ | 10 | L: Nothing found; spelling unclear from sound (SPOO-day) | 5 | 5 | 4 | 2 | **4.30** |
| 4 | **Dirasa** | Arabic · دِرَاسَة | study, studies | 6 | `dira` | ✓ ✓ ✓ | 52 | M: No product found, but it is the everyday Arabic word for "study" (like Studium in German), so hard to own in Arabic-speaking markets | 3.5 | 5 | 5 | 3 | **4.17** |
| 5 | **Kepos** | Greek · κῆπος | garden; Epicurus's school, "the Garden" | 5 | `kep` | ✓ ✓ ✓ | 51 | L: Nothing found; "the Garden" is less about study | 4.5 | 5 | 3 | 4 | **4.17** |
| 6 | **Prokope** | Greek · προκοπή | progress; the Stoic term for steady progress toward mastery | 7 | `prok` | ✓ ✓ ✓ | 143 | L: Nothing found; may be read as "pro-cope" | 4 | 4 | 5 | 3 | **4.10** |
| 7 | **Tanren** | Japanese · 鍛錬 | forging; disciplined training toward mastery (Musashi) | 6 | `tan` | ✓ ✗ ✗ | 42 | L: Nothing found; npm and PyPI taken | 3.5 | 5 | 4 | 4 | **4.07** |
| 8 | **Lychnos** | Greek · λύχνος | lamp; "smells of the lamp" was said of hard-worked study (Demosthenes) | 7 | `lyc` | ✓ ✓ ✓ | 5 | L: Nothing found; "ch" makes it hard to spell from sound | 5 | 4 | 4 | 2 | **4.05** |
| 9 | **Manthano** | Greek · μανθάνω | I learn | 8 | `mant` | ✓ ✓ ✓ | 34 | L: not web-checked; `man` alias would clash, so `mant` | 4.5 | 3.5 | 4 | 3 | **3.90** |
| 10 | **Melete** | Greek · μελέτη | study, practice, care; one of the three original Muses (Melete, Muse of practice) | 6 | `mel` | ✗ ✓ ✓ | 81 | M: Meleti study app on a pre-launch waitlist (modern-Greek spelling of the same word); crate taken | 2.5 | 5 | 5 | 3 | **3.83** |
| 11 | **Dhakira** | Arabic · ذَاكِرَة | memory | 7 | `dha` | ✓ ✗ ✓ | 24 | -: not web-checked; npm taken | 4.0 | 4.5 | 4 | 2 | **3.82** |
| 12 | **Tahsil** | Arabic · تَحْصِيل | attainment, acquisition of knowledge | 6 | `tah` | ✓ ✓ ✓ | 195 | M: Nothing found; but "tehsil" is a South Asian administrative district | 3 | 5 | 4 | 3 | **3.75** |
| 13 | **Kherad** | Persian · خرد | wisdom, intellect | 6 | `khe` | ✓ ✓ ✓ | 61 | L: Nothing found | 4.5 | 4.5 | 3 | 2 | **3.75** |
| 14 | **Qandil** | Arabic · قِنْدِيل | lamp, lantern | 6 | `qan` | ✓ ✓ ✓ | 157 | L: Nothing found | 4 | 5 | 3 | 2 | **3.70** |
| 15 | **Ijaza** | Arabic · إِجَازَة | licence to transmit knowledge, issued by a master to a student; a forerunner of the degree | 5 | `ija` | ✓ ✓ ✗ | 362 | M: No app found, but strongly tied to Quran ijazah certification; also means "holiday/permission" in modern Arabic | 2.5 | 4.5 | 5 | 3 | **3.70** |
| 16 | **Akribeia** | Greek · ἀκρίβεια | precision, exactness | 8 | `akr` | ✓ ✓ ✓ | 7 | L: Nothing found | 5 | 3.5 | 3 | 2 | **3.67** |
| 17 | **Ustad** | Arabic/Persian · أُسْتَاذ | master; professor | 5 | `ust` | ✓ ✓ ✓ | 729 | M: not web-checked; common title; tutoring brands | 2.5 | 4.5 | 4 | 4 | **3.60** |
| 18 | **Mudhakara** | Arabic · مُذَاكَرَة | studying, revising (everyday word for exam study in many Arabic dialects) | 9 | `mud` | ✓ ✓ ✓ | 6 | -: not web-checked; many spellings (mudhakara/muzakra) | 4.5 | 2.5 | 5 | 1 | **3.60** |
| 19 | **Halaqa** | Arabic · حَلْقَة | study circle | 6 | `hal` | ✓ ✓ ✓ | 358 | M: Islamic study circles; community apps | 3 | 5 | 4 | 2 | **3.60** |
| 20 | **Dysg** | Welsh · dysg | to learn; learning | 4 | `dysg` | ✓ ✓ ✓ | 213 | M: Dysgu (Welsh "to learn") research system; looks like a typo to non-Welsh readers | 3 | 5 | 4 | 2 | **3.60** |
| 21 | **Fahm** | Arabic · فَهْم | understanding | 4 | `fahm` | ✓ ✓ ✓ | 3,996 | -: not web-checked; looks like "farm"/"fam" to English readers | 2.5 | 5 | 4 | 3 | **3.58** |
| 22 | **Takrar** | Arabic · تَكْرَار | repetition | 6 | `tak` | ✓ ✓ ✓ | 7 | H: Tikrar Quran repetition app | 2.5 | 5 | 4 | 3 | **3.58** |
| 23 | **Anodos** | Greek · ἄνοδος | the way up, ascent | 6 | `ano` | ✓ ✓ ✓ | 33 | H: Anodos: children's retention/spaced-learning app (2026) | 2.0 | 5 | 4 | 4 | **3.55** |
| 24 | **Alim** | Arabic · عَالِم | scholar, learned person | 4 | `alim` | ✓ ✓ ✓ | 17,518 | M: not web-checked; religious-scholar connotation; common name | 2 | 5 | 4 | 4 | **3.55** |
| 25 | **Epimeleia** | Greek · ἐπιμέλεια | diligence, care; "care of the self" | 9 | `epi` | ✓ ✓ ✓ | 3 | L: not web-checked; 9 letters, hard to spell | 5 | 2.5 | 4 | 1 | **3.52** |
| 26 | **Agoge** | Greek · ἀγωγή | training, formation | 5 | `ago` | ✓ ✓ ✗ | 93 | M: Fitness brands (Spartan agoge) | 3.0 | 5 | 3 | 3 | **3.50** |
| 27 | **Hexis** | Greek · ἕξις | a stable disposition acquired through practice (Aristotle) | 5 | `hex` | ✗ ✓ ✗ | 296 | H: Hexis: local-first desktop app for tasks, goals, habits and timers (direct competitor) | 1 | 5 | 5 | 4 | **3.45** |
| 28 | **Hashiya** | Arabic · حَاشِيَة | marginal gloss, commentary | 7 | `has` | ✓ ✓ ✓ | 30 | -: not web-checked | 4.0 | 4 | 3 | 2 | **3.45** |
| 29 | **Anagnosis** | Greek · ἀνάγνωσις | reading; recognition | 9 | `ana` | ✓ ✓ ✓ | 18 | L: not web-checked; sounds medical | 5 | 2.5 | 3 | 2 | **3.42** |
| 30 | **Elimu** | Swahili · elimu | education, knowledge | 5 | `eli` | ✓ ✓ ✓ | 841 | M: not web-checked; common in Swahili edtech | 2.5 | 4.5 | 3 | 4 | **3.35** |
| 31 | **Muraja** | Arabic · مُرَاجَعَة | review, revision (also the term for revising memorised text) | 6 | `mur` | ✓ ✓ ✓ | 103 | H: Quran revision apps (Murajaa, Muraajah Map); apostrophe spellings vary | 1.5 | 5 | 5 | 2 | **3.32** |
| 32 | **Itqan** | Arabic · إِتْقَان | mastery; doing something to perfection | 5 | `itq` | ✓ ✓ ✗ | 629 | H: Several Quran memorisation apps (Itqan Al-Qur'an, Itqaan, Itqān Studio) | 1 | 5 | 5 | 3 | **3.30** |
| 33 | **Rihla** | Arabic · رِحْلَة | journey; "rihla fi talab al-ilm", the scholar's journey in search of knowledge | 5 | `rih` | ✓ ✓ ✗ | 537 | H: Rihla: Moroccan Ministry of Education revision app | 1 | 5 | 5 | 3 | **3.30** |
| 34 | **Malaka** | Arabic · مَلَكَة | acquired faculty, mastery through practice (Ibn Khaldun) | 6 | `mlk` | ✓ ✗ ✓ | 1,078 | H: Disqualified: vulgar slang in Greek (μαλάκας) | 1 | 5 | 5 | 3 | **3.30** |
| 35 | **Danesh** | Persian · دانش | knowledge | 6 | `dan` | ✓ ✗ ✓ | 1,380 | M: Nothing found; common given name | 2.0 | 5 | 3 | 4 | **3.30** |
| 36 | **Gnothi** | Greek · γνῶθι | know! (from "know thyself") | 6 | `gno` | ✓ ✓ ✓ | 28 | H: Gnothi: AI self-reflection/journal app | 2.0 | 5 | 4 | 2 | **3.25** |
| 37 | **Dars** | Arabic · دَرْس | a lesson; study | 4 | `dars` | ✗ ✗ ✗ | 59,992 | H: All registries taken; very common | 1 | 4.5 | 4 | 5 | **3.23** |
| 38 | **Marifa** | Arabic · مَعْرِفَة | knowledge, knowing | 6 | `mar` | ✓ ✓ ✓ | 160 | H: Maarifa Islamic AI assistant app | 1.5 | 5 | 4 | 3 | **3.22** |
| 39 | **Benkyo** | Japanese · 勉強 | study | 6 | `ben` | ✓ ✓ ✗ | 793 | M: not web-checked; many Japanese study apps | 2.0 | 4.5 | 3 | 4 | **3.18** |
| 40 | **Talab** | Arabic · طَلَب | seeking (from talab al-ilm, seeking knowledge) | 5 | `tlb` | ✓ ✓ ✓ | 3,044 | M: not web-checked; risk of unwanted associations in English | 2 | 5 | 3 | 3 | **3.15** |
| 41 | **Philomath** | Greek · φιλομαθής | lover of learning | 9 | `phil` | ✓ ✓ ✗ | 232 | M: not web-checked; established English word; Philomath, Oregon | 2.5 | 1.5 | 5 | 4 | **3.10** |
| 42 | **Midad** | Arabic · مِدَاد | ink; the scholar's ink of the proverb ("the ink of scholars…") | 5 | `mid` | ✓ ✗ ✓ | 312 | H: Midad Smart Madrasa teaching app | 1 | 4.5 | 4 | 4 | **3.08** |
| 43 | **Mouseion** | Greek · Μουσεῖον | the Mouseion of Alexandria, the ancient research institute | 8 | `mou` | ✓ ✗ ✗ | 60 | M: not web-checked; npm and PyPI taken; reads as "mouse" | 2.5 | 2.5 | 5 | 2 | **3.05** |
| 44 | **Autonomia** | Greek · αὐτονομία | self-governance, autonomy | 9 | `aut` | ✓ ✓ ✓ | 396 | M: Generic word in several languages; Italian political movement | 3 | 2 | 3 | 5 | **3.05** |
| 45 | **Iyun** | Hebrew · עִיּוּן | deep, careful study | 4 | `iyu` | ✓ ✗ ✓ | 630 | -: not web-checked | 2.5 | 4.5 | 3 | 2 | **3.05** |
| 46 | **Svadhyaya** | Sanskrit · स्वाध्याय | self-study | 9 | `sva` | ✓ ✓ ✓ | 43 | -: not web-checked | 4.0 | 2 | 4 | 1 | **3.05** |
| 47 | **Ijtihad** | Arabic · اِجْتِهَاد | independent scholarly reasoning; diligent effort | 7 | `ijt` | ✓ ✓ ✓ | 52 | H: Religious-legal term; sensitive positioning | 2.0 | 4 | 4 | 2 | **3.00** |
| 48 | **Enkrateia** | Greek · ἐγκράτεια | self-mastery, self-command | 9 | `enk` | ✓ ✓ ✗ | 18 | L: not web-checked | 4.5 | 2 | 3 | 1 | **2.98** |
| 49 | **Opi** | Finnish · opi | learn! | 3 | `opi` | ✗ ✗ ✓ | 24,872 | H: OPI nail-polish brand; crates and npm taken | 1 | 4.5 | 3 | 5 | **2.98** |
| 50 | **Siraj** | Arabic · سِرَاج | lamp | 5 | `sir` | ✓ ✗ ✓ | 4,097 | H: Siraj Learning, Siraj Teacher and several Quran apps | 1 | 5 | 3 | 4 | **2.95** |
| 51 | **Mahara** | Arabic · مَهَارَة | skill | 6 | `mah` | ✓ ✓ ✓ | 5,458 | H: Mahara: open-source ePortfolio platform used by universities (knowledge) | 1 | 5 | 3 | 4 | **2.95** |
| 52 | **Abhyasa** | Sanskrit · अभ्यास | steady practice | 7 | `abh` | ✓ ✓ ✗ | 150 | M: not web-checked; yoga apps | 2.5 | 4 | 3 | 2 | **2.92** |
| 53 | **Philoponia** | Greek · φιλοπονία | love of hard work, industriousness | 10 | `pho` | ✓ ✓ ✓ | 0 | L: not web-checked | 5 | 1 | 3 | 1 | **2.90** |
| 54 | **Soma** | Swahili · soma | read; study | 4 | `som` | ✗ ✗ ✗ | 22,599 | H: Drug and many brands (knowledge) | 1 | 5 | 2 | 5 | **2.85** |
| 55 | **Daab** | Arabic · دَأْب | perseverance, constant diligence | 4 | `daab` | ✗ ✗ ✗ | 408 | H: All registries taken | 1 | 4.5 | 3 | 2 | **2.52** |
| 56 | **Phronesis** | Greek · φρόνησις | practical wisdom | 9 | `phr` | ✗ ✗ ✗ | 336 | H: All registries taken; common in philosophy/consulting | 1 | 2.5 | 4 | 2 | **2.27** |

## 14. Sources for round-2 web checks

- Hexis (desktop productivity app): <https://www.producthunt.com/products/hexis>
- Anodos: <https://apps.apple.com/pl/app/anodos/id6757709245>
- Meleti (waitlist): <https://meleti-waitlist.pages.dev>
- Itqan Al-Qur'an: <https://apps.apple.com/se/app/itqan-al-quran/id1505510107>
- Itqaan – Quran AI: <https://apps.apple.com/app/id6759169237>
- Murajaa / Muraajah Map: <https://appadvice.com/app/muraajah-map/6480593479>
- Tikrar (Quran repetition): <https://mwm.ai/apps/app/1498348815>
- Rihla (Moroccan Ministry of Education): <https://lematin.ma/enseignement/rihla-lapplication-gratuite-pour-aider-les-eleves-du-primaire-a-reviser/360041>
- Midad Smart Madrasa: <https://mwm.ai/apps/midad-smart-madrasa/6780352177>
- Siraj Learning: <https://apps.apple.com/us/app/id1593340498>
- Maarifa Islam AI: <https://apps.apple.com/app/id6572303135>
- Gnothi: <https://www.producthunt.com/p/gnothi-2>
- Dysgu (research system): <https://sourceforge.net/p/dysgu>

## 15. Round 3: developer brief (2026-11-22)

The pivot to developers and CS students (D-027) changed who judges the name. Round 3 re-ran the checks against a developer brief, with the round-1 and round-2 favourites kept in as the names to beat.

### 15.1 Brief

- **Meaning:** memory, recall or keeping what you learn ("Keep what you learn while you code"), ideally with a developer flavour.
- **CLI:** a binary of 6 letters or fewer and an alias of 2–4. Neither may already be a command, a Homebrew formula or cask, or a Debian package.
- **Uniqueness:** free on crates.io, npm and PyPI; low GitHub noise; no developer tool, flashcard app or memory app with the same name.
- **Domains:** at least one of `.dev`, `.sh`, `.io`, `.app` free.

### 15.2 Method

- **44 candidates** in four groups: the incumbents (Addisco, Edisco, Lampas); memory words from Latin, Greek, Norse and neuroscience (Memini, Mneme, Engram, Muninn, Loci…); craft metaphors for marking and keeping (Etch, Cairn, Latch, Kept, Ingrain…); and coined, developer-style forms (Spacd, Recue, Stickr, Keepr…).
- **Automated checks, all run on 2026-11-22:**
  - crates.io, npm and PyPI;
  - Homebrew formulae and casks, and Debian source packages (as a proxy for Linux commands);
  - commands on a macOS developer machine, for both the name and the alias;
  - GitHub repositories with the word in their name, and whether the GitHub user or org handle exists;
  - DNS for `.com`, `.dev`, `.sh`, `.io` and `.app`.

  A domain with no NS or SOA record is *probably* unregistered. That's a signal, not proof.
- **Web checks** for the 15 strongest names, looking for developer tools, flashcard and memory apps.

**Scoring:**

| Criterion | Weight | How it's scored |
|---|---|---|
| Uniqueness | 35% | As in §3 |
| CLI fit | 30% | ≤5 letters = 5, 6 = 4.5, 7 = 4, 8 = 3, ≥9 = 2. −1 if the name is already a command or package; −0.5 if the alias is. ±0.5 for good/poor QWERTY hand alternation. |
| Meaning | 20% | Fit with "keep what you learn while you code" (judgement) |
| Say & spell | 15% | As in §3 |

### 15.3 Results (top 12)

| # | Name | Command / alias | Score | Registries | GitHub repos | Web | Domains free (DNS) | Verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | **Edisco** | `edisco` / `edi` | 4.33 | all free | 235 (mostly *ediscovery*) | L | `.dev`, `.sh` | Best meaning ("I learn by heart") and 6 letters. But search noise from e-discovery, `edi` reads as EDI (electronic data interchange) to developers, and `.com`, `.io` and `.app` are taken. |
| 2 | **Addisco** | `addisco` / `addi` | 4.10 | all free | **39** | L | **`.dev`, `.sh`, `.io`, `.app`** | The cleanest name in every check, and no cost to keep. The meaning ("I keep learning") matches the new tagline. 7 letters; "add + disco" reading. |
| 3 | Lampas | `lampas` / `lam` | 4.10 | all free | 88 | L | `.sh`, `.app` | Weak meaning for this brief (a torch, not memory). `lam` is a command and a Debian package. |
| 4 | **Memini** | `memini` / `mem` | 4.07 | npm, PyPI taken | 90 | L | `.sh` | Latin "I remember": the best meaning among the new names. A dead wearable camera "meMINI" had a "Recall" button. `mem` is a generic alias. |
| 5 | Spacd | `spacd` / `sp` | 3.88 | all free | 72 | – | `.sh` | Coined from "spaced"; hard to spell from hearing |
| 6 | Ingrain | `ingrain` / `ing` | 3.80 | npm, PyPI taken | 60 | – | `.sh` | Good meaning and easy to say; a common English word |
| 7 | Recue | `recue` / `rq` | 3.67 | all free | 1,838 | – | `.sh`, `.app` | Reads as "rescue" |
| 8 | Respace | `respace` / `rsp` | 3.62 | npm, PyPI taken | 99 | – | `.sh` | Descriptive but flat |
| 9 | Teneo | `teneo` / `ten` | 3.60 | npm taken | 372 | M | `.dev` | Teneo is a large consultancy (knowledge) |
| 10 | Stickr | `stickr` / `sti` | 3.60 | npm taken | 227 | – | `.dev`, `.sh` | 2010s-style dropped vowel; hard to say what's spelled |
| 11 | Reken | `reken` / `rek` | 3.57 | all free | 1,731 | – | `.sh`, `.io` | Dutch for "calculate" |
| 12 | Ebbing | `ebbing` / `ebb` | 3.55 | npm taken | 360 | – | `.com`, `.sh` | A nod to Ebbinghaus, but it's about forgetting |

### 15.4 What round 3 taught us

- **The best developer-flavoured names are taken by AI-memory tools.**
  - *Engram*: at least six MCP and agent-memory projects (Go, Rust, npm), plus "Engram: Memory Training", an FSRS flashcard app.
  - *Muninn*: an AI memory layer launched on Product Hunt in February 2026, and a developer knowledge tool.
  - *Cairn*: several local-first Markdown and second-brain tools.

  Memory for AI agents became a crowded category in 2026.
- **Short English words are gone on every registry.** Etch, Latch, Kept, Rote, Recur, Grasp, Tally, Lore and Ebb all have crates, npm and PyPI packages, and thousands of GitHub repos. *Etch* is also two flashcard apps (Etch Memory, EtchNote).
- **Memory-themed coinages collide with study apps.**
  - Mnemo is a free FSRS desktop study app;
  - Memor More is a spaced-repetition app;
  - Remem sits between RemNote and "Remember..?";
  - Loci is used by several memory-palace apps.
- **GitHub handles are almost all taken.** All 44 names but one (Codecard) exist as GitHub users or orgs, including all four finalists. Plan for an org name such as `addisco-dev` or `addiscohq`, whichever name is chosen.
- **The Latin incumbents are still the scarcest names that mean something.** That's why they lead round 3 too.

### 15.4a Batch 2: programming puns and compounds

The first batch had few names with a real developer feel, so a second batch of 30 was checked the same way. It covered puns on programming terms (Memoize, Keepalive, Rehash, Reflog, Memheap, Mempin, Memit), daemon and `-ctl` styles (Recalld, Memctl, Memsh) and compounds (Codekeep, Gitrecall, Codedrill).

| # | Name | Command / alias | Score | Registries | GitHub | Web | Domains free (DNS) | Verdict |
|---|---|---|---|---|---|---|---|---|
| 1 | Memheap | `memheap` / `mh` | 4.23 | all free | 2 | L | **all five, incl. `.com`** | The only name in either batch whose `.com` looks free. But to developers a "mem heap" is an allocator or a heap profiler (RT-Thread has a `memheap` allocator), so it misdescribes the product. |
| 2 | Mempin | `mempin` / `mp` | 4.03 | all free | 19 | M | `.dev`, `.sh`, `.io`, `.app` | "Pin it in memory": pinned memory is a real programming concept. MemPin is also an AI memory-manager app for iPad and visionOS. Same misreading risk as Memheap. |
| 3 | Memit | `memit` / `mit` | 4.02 → **3.50** | all free | 126 | **H** | `.dev`, `.sh`, `.app` | The best pun ("commit it to memory", "mem it"). The flashcard web app with the name is dead (0 stars, last commit 2019, memit.ca doesn't resolve), and an older "Memit" service is discontinued. But developers already know the name for two things. **MEMIT**, "Mass-Editing Memory in a Transformer" (ICLR 2023, 570★), edits facts stored in language models: the same "memory" space. **liamg/memit** (319★) is a Linux CLI with the same command name. `.com` and `.io` are registered; `mit` reads as MIT. |
| 4 | Rehash | `rehash` / `rh` | 3.92 | npm, PyPI taken | 347 | – | `.sh`, `.app` | `rehash` is a zsh built-in |
| 5 | Keepcode | `keepcode` / `kc` | 3.90 | all free | 67 | – | `.sh`, `.io` | Too close to CodeKeep, a developer snippet manager |

Also out:
- **Codekeep**: the CodeKeep snippet manager.
- **Memoize/Memoise**: taken on every registry, and a standard library term.
- **Keepalive**: 4,246 GitHub repos; a TCP term.
- **Kommit**: KDE's Git client.
- **Reflog**: a Git subcommand.
- **Unforget**: npm and PyPI taken; no domains free.

**What batch 2 taught us:** developer-pun names *are* available, and Memheap even has `.com`. But a pun on a programming term makes a developer expect *that* kind of tool: a heap profiler, a memory pinner, a cache. For a learning tool that misdirection costs more than an unfamiliar Latin word does. The pun works better in the tagline and commands (`addi while -- cargo test`, "commit it to memory") than in the name.

### 15.5 Recommendation: keep Addisco

Edisco scores slightly higher (4.33 vs 4.10), on length and meaning. For a developer product, three things the score doesn't capture favour Addisco:

0. **The developer-pun names mislead or collide** (§15.4a): Memheap and Mempin read as low-level memory tools. Memit leads developers to the MEMIT language-model editing paper and to an existing `memit` CLI.
1. **Searchability.** Developers find tools by searching. "Edisco" collides with *e-discovery* (235 GitHub repos), and its alias `edi` collides with *EDI*. Addisco has the fewest GitHub repos of any candidate (39) and no products found.
2. **Domains.** `addisco.dev`, `.io`, `.app` and `.sh` all look free. For Edisco only `.dev` and `.sh` do.
3. **Cost.** Keeping the name costs nothing. A rename touches the spec and the 0.1 alpha binaries already on GitHub, for a marginal gain.

The meaning also still works: *addisco*, "I keep learning, I add to what I know", sits naturally under "Keep what you learn while you code". The developer positioning comes from the tagline, the demo and `addi while -- cargo test`, not from the word.

**If you'd rather change:** Edisco is the alternative. Memini is a distant third, held back by npm and PyPI, and by `.sh` being its only free domain.

### 15.6 Do now, whichever name

- [ ] Register `addisco.dev` (and `.io` and `.app` if the budget allows). `addisco.com` is already registered, so check who holds it during the trademark search.
- [ ] Reserve `addisco` on crates.io, the `@addisco` npm scope and a GitHub org (`addisco-dev` or similar; the `addisco` user handle has belonged to an inactive account since 2015).
- [ ] Pick the Homebrew tap name to match the org.
- [ ] Trademark search in classes 9, 41 and 42 (§7).
- [ ] Say it aloud to 5–10 developers and CS students during the M0.6 interviews.

### 15.7 Sources (round 3)

- Engram: [Gentleman-Programming/engram](https://pkg.go.dev/github.com/Gentleman-Programming/engram) · [engram-cli](https://pi.dev/packages/engram-cli?page=71) · [engramai](https://docs.rs/crate/engramai/latest/source/INTEGRATION.md) · [Engram: Memory Training](https://apps.apple.com/us/app/engram-memory-training/id6784637263)
- Muninn: [Product Hunt](https://www.producthunt.com/posts/muninn) · [Erebuzzz/Muninn](https://github.com/Erebuzzz/Muninn/wiki) · [ytisf/muninn](https://ytisf.github.io/muninn/)
- Cairn: [ddutchie/cairn](https://ddutchie.github.io/cairn-site/) · [OpenCairn](https://www.producthunt.com/products/opencairn) · [p10q/cairn](https://github.com/p10q/cairn)
- Etch: [Etch Memory](https://apps.apple.com/us/app/etch-memory/id6741780207) · [EtchNote](https://apps.apple.com/us/app/id6759913456)
- Mnemo and Memor: [Mnemo](https://alternativeto.net/software/mnemo/about/) · [Mnemesia](https://mwm.ai/apps/mnemesia/6812848180) · [Memor More](https://apps.apple.com/mx/app/memor-more/id6757725097)
- Loci: [Lociform](https://apps.apple.com/app/id6800037106) · [Mind Palace](https://apps.apple.com/app/id6758350669)
- Remem: [Remember..?](https://apps.apple.com/ly/app/remember/id1587047424) · [Rememba](https://apps.apple.com/se/app/rememba-app/id1033305988)
- Memini: [meMINI (CB Insights)](https://www.cbinsights.com/company/memini) · [Memnio](https://apps.apple.com/om/app/memnio/id6502569622)
- Memit: [MEMIT paper](https://arxiv.org/abs/2210.07229) · [kmeng01/memit](https://github.com/kmeng01/memit) · [liamg/memit](https://github.com/liamg/memit) · [russlbrown/memit](https://github.com/russlbrown/memit) · [Memit (discontinued)](https://alternativeto.net/software/memit/about) · Mempin: [MemPin app](https://apps.apple.com/us/app/mempin/id6742840746)
- Addisco and Edisco: no matching software products found ([search 1](https://www.adiscon.com/?p=57), [search 2](https://apps.apple.com/pl/app/velamp-tools/id1466815060) returned only near-names)
