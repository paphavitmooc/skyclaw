# Blueprint Matching v2 — Token-Aware, Zero Extra LLM Calls

## Problem Statement

Blueprint matching v1 made a dedicated LLM call to pick the best blueprint for each compound task. This added latency and cost. Worse, the memory backend (`memory.search()`) was just SQL `LIKE '%word%'` — no real semantic understanding. And the original scoring used brittle `string.contains()` matching against LLM-authored trigger patterns.

Additionally, blueprints could be arbitrarily large. A complex deployment procedure might be 10K+ tokens. Injecting it blindly could overflow the context window, crowd out conversation history, or cause the model to loop.

## Design Philosophy

**The LLM is a thinking brain with finite working memory.** The context window is the skull — hard limit, no exceptions.

1. Every resource declares its cost — measured once, stored in metadata
2. The brain sees its own budget — Resource Budget Dashboard in system prompt
3. Graceful degradation over failure — truncate before reject, reject before crash
4. The brain decides priorities — it sees the catalog with costs
5. Upstream hints feed downstream — no extra LLM calls, leverage existing ones
6. Grounded vocabularies — classifier picks from actual stored categories, never invents

## Architecture

### Flow

```
1. Pre-classifier: fetch distinct semantic_tags from stored blueprints
   → e.g. ["code-analysis", "disk-ops", "deployment"]
   → Pure SQL metadata query, no LLM

2. Classifier (existing LLM call, +1 field):
   Input:  user message + history + "pick from [code-analysis, disk-ops, deployment] or null"
   Output: { category, chat_text, difficulty, blueprint_hint: "code-analysis" }
   Cost:   ~0 extra tokens (one short field added)

3. Fetch blueprints by category:
   SELECT FROM memory_entries WHERE entry_type='blueprint'
   Parse each, filter where semantic_tags contains blueprint_hint
   → Returns 0-N blueprints with pre-computed token counts

4. Build context with blueprint awareness:
   a. Inject Resource Budget Dashboard into system prompt
   b. If blueprints found: inject compact catalog (name + description + token cost)
   c. Auto-load best blueprint body if it fits within 10% budget
   d. If too large: inject outline (objective + phase headers only)

5. Main LLM runs (existing tool loop, no extra calls):
   - Sees budget dashboard, knows its limits
   - Sees blueprint catalog and/or loaded blueprint body
   - Follows blueprint or ignores it — its choice

6. Post-task (background, existing):
   - Blueprint authoring or refinement, same as before
   - Token count computed and stored in metadata at authoring time
```

### Grounded Categories

The classifier MUST pick from actually stored categories. This prevents mismatches:

```
Stored blueprints have tags: ["code-analysis", "disk-ops"]
Classifier prompt: blueprint_hint: pick from ["code-analysis", "disk-ops"] or null
Classifier output: blueprint_hint: "code-analysis"  ← exact match guaranteed
```

If no blueprints exist, `blueprint_hint` field is omitted from the classifier prompt entirely. Zero overhead for new users.

### Resource Budget Dashboard

Injected into system prompt on every context rebuild:

```
=== CONTEXT BUDGET ===
Model: gpt-5.2 | Limit: 128,000 tokens
Used: 14,200 tokens
  System:     2,100
  Tools:      3,400 (13 definitions)
  Blueprint:  1,247 (loaded: bp-async-scan)
  Memory:     1,200
  Learnings:    450
  History:    5,803 (12 turns)
Available: 113,800 tokens
Blueprint budget remaining: 11,553 / 12,800
=== END BUDGET ===
```

### Blueprint Catalog Format

```
=== AVAILABLE BLUEPRINTS ===
To follow a blueprint, state "I'll follow blueprint [id]" in your response.
  [bp-async-scan] (1,247 tok) Scan Rust workspace for async fn per crate
  [bp-disk-report] (2,103 tok) Disk usage + cleanup report (macOS)
  * bp-async-scan auto-loaded (best match for category "code-analysis")
=== END BLUEPRINTS ===
```

### Token Accounting

| Resource | Measured when | Stored where |
|----------|-------------|-------------|
| Blueprint body | At authoring/refinement | `metadata.token_count` in memory entry |
| Tool definition | At registration | Computed in `build_context()` |
| System prompt | Every context rebuild | Computed in `build_context()` |
| History | Every context rebuild | Computed in `build_context()` |
| Model context limit | At startup | From model registry |

### Budget Enforcement

| Scenario | Action |
|----------|--------|
| Best blueprint fits in 10% budget | Auto-inject full body |
| Best blueprint > 10% budget but < 25% | Inject outline (objective + phase headers) |
| Best blueprint > 25% of total context | Reject, show only catalog |
| Catalog itself > 500 tokens (many blueprints) | Cap catalog at top 5 by success rate |
| No blueprint_hint from classifier | No catalog, no fetch, zero overhead |
| blueprint_hint but no matching blueprints | Catalog section omitted |
| Context shrinking across turns | Dashboard updates, LLM sees budget decreasing |

### Resilience

- **Classifier fails**: Falls back to rule-based classification (existing). `blueprint_hint` is None. No blueprints loaded. Agent works normally.
- **Category fetch fails (DB error)**: Log warning, continue without blueprints. Agent works normally.
- **Blueprint parsing fails**: Skip that entry, log debug. Continue with remaining candidates.
- **Token estimation is inaccurate**: 10% safety margin built into budget calculations. Budget dashboard shows conservative estimates.
- **Blueprint body corrupted**: `parse_blueprint()` returns Err, entry skipped. Agent works normally.
- **All blueprints too large**: Catalog shown but no body loaded. Agent works without blueprint guidance. May create a new (hopefully smaller) blueprint after task completion.
- **Model has tiny context (8K)**: 10% = 800 tokens. Most blueprints won't fit. Catalog still shown (cheap). Agent degrades gracefully.

## Implementation Steps

### Step 1: Add token_count to Blueprint

Add `token_count: usize` to `Blueprint` struct. Computed at authoring time via `estimate_tokens()`. Stored in memory entry metadata.

### Step 2: Add fetch_available_categories()

New function in `blueprint.rs`: queries all blueprint entries from memory, parses metadata, returns deduplicated `Vec<String>` of all semantic_tags.

### Step 3: Add fetch_by_category()

New function in `blueprint.rs`: fetches all blueprint entries from memory, parses each, filters where `semantic_tags` contains the target category. Returns `Vec<Blueprint>` sorted by success_rate descending.

### Step 4: Update classifier

Add `blueprint_hint` field to prompt and response format. Pass available categories as allowed values. Field is `null` for chat/stop messages and when no blueprints exist.

### Step 5: Format catalog and budget dashboard

New functions in `blueprint.rs`:
- `format_blueprint_catalog(blueprints, loaded_id)` → catalog string
- New in `context.rs`: `format_budget_dashboard(allocations)` → dashboard string

### Step 6: Update build_context()

- Accept `Vec<Blueprint>` (matched by category) instead of `Option<Blueprint>`
- Auto-select best blueprint for body injection (highest success_rate, fits budget)
- Inject catalog section after tool definitions
- Inject budget dashboard into system prompt
- Track all allocations in a struct

### Step 7: Update runtime

- Fetch categories before classifier call
- Pass categories to classifier
- Pass `blueprint_hint` to category fetch
- Pass matched blueprints to `build_context()`
- Remove old `find_matching_blueprint()` LLM call

### Step 8: Update authoring

- Compute `token_count` via `estimate_tokens()` at authoring time
- Store in metadata alongside existing fields
- Update on refinement

### Step 9: Tests

- Classifier tests with `blueprint_hint` field
- Category fetch tests
- Catalog formatting tests
- Budget dashboard tests
- Token counting at authoring time
- Budget enforcement (too large, outline, reject)

## Files Changed

| File | Change |
|------|--------|
| `blueprint.rs` | Remove LLM matching. Add `token_count`, `fetch_available_categories()`, `fetch_by_category()`, `format_blueprint_catalog()`, `format_blueprint_outline()`. Update `to_memory_entry()` to store token_count. |
| `llm_classifier.rs` | Add `blueprint_hint` field, update prompt with available categories. |
| `context.rs` | Add budget dashboard. Change blueprint injection from `Option<Blueprint>` to `Vec<Blueprint>` with auto-selection and budget enforcement. |
| `runtime.rs` | Fetch categories pre-classifier. Remove LLM matching call. Wire category fetch → context builder. |

## Risk Assessment

**ZERO RISK** — All changes are additive or replace existing blueprint matching code introduced in this same feature branch. No existing user-facing behavior changes. The classifier prompt change adds an optional field that degrades gracefully (serde `#[serde(default)]`). Context builder changes only affect the new blueprint code path. All existing tests continue to pass.
