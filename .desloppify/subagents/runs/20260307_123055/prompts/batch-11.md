You are a focused subagent reviewer for a single holistic investigation batch.

Repository root: E:\SpedImage
Blind packet: E:\SpedImage\.desloppify\review_packet_blind.json
Batch index: 11
Batch name: design_coherence
Batch rationale: seed files for design_coherence review

DIMENSION TO EVALUATE:

## design_coherence
Are structural design decisions sound — functions focused, abstractions earned, patterns consistent?
Look for:
- Functions doing too many things — multiple distinct responsibilities in one body
- Parameter lists that should be config/context objects — many related params passed together
- Files accumulating issues across many dimensions — likely mixing unrelated concerns
- Deep nesting that could be flattened with early returns or extraction
- Repeated structural patterns that should be data-driven
Skip:
- Functions that are long but have a single coherent responsibility
- Parameter lists where grouping would obscure meaning
- Files that are large because their domain is genuinely complex, not because they mix concerns
- Nesting that is inherent to the problem (e.g., recursive tree processing)

YOUR TASK: Read the code for this batch's dimension. Judge how well the codebase serves a developer from that perspective. The dimension rubric above defines what good looks like. Cite specific observations that explain your judgment.

Mechanical scan evidence — navigation aid, not scoring evidence:
The blind packet contains `holistic_context.scan_evidence` with aggregated signals from all mechanical detectors — including complexity hotspots, error hotspots, signal density index, boundary violations, and systemic patterns. Use these as starting points for where to look beyond the seed files.

Seed files (start here):
- src/gpu_renderer.rs
- src/app/events.rs
- src/app/actions.rs
- src/image_backend.rs
- src/app/state.rs
- src/ui.rs
- src/lib.rs
- src/main.rs
- src/app/services.rs
- src/app/types.rs

Mechanical concern signals — navigation aid, not scoring evidence:
Confirm or refute each with your own code reading. Report only confirmed defects.
  - [design_concern] src/app/services.rs
    summary: Design signals from orphaned
    question: Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned
    evidence: [orphaned] Orphaned file (142 LOC): zero importers, not an entry point
  - [design_concern] src/app/state.rs
    summary: Design signals from orphaned, signature
    question: Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned, signature
    evidence: [signature] 'new' has 3 different signatures across 3 files
  - [design_concern] src/lib.rs
    summary: Design signals from orphaned
    question: Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned
    evidence: [orphaned] Orphaned file (13 LOC): zero importers, not an entry point
  - [design_concern] src/main.rs
    summary: Design signals from orphaned
    question: Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned
    evidence: [orphaned] Orphaned file (108 LOC): zero importers, not an entry point
  - [interface_design] src/gpu_renderer.rs
    summary: Interface complexity: 12 parameters
    question: Should the parameters be grouped into a config/context object? Which ones belong together? Can the nesting be reduced with early returns, guard clauses, or extraction into helper functions? Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned, structural
    evidence: File size: 1560 lines
  - [mixed_responsibilities] src/ui.rs
    summary: Issues from 2 detectors — may have too many responsibilities
    question: Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)? What are the distinct responsibilities? Would splitting produce modules with multiple independent consumers, or would extracted files only be imported by the parent? Only split if the extracted code would be reused.
    evidence: Flagged by: orphaned, responsibility_cohesion
    evidence: [responsibility_cohesion] 6 disconnected function clusters (23 functions) — likely mixed responsibilities
  - [structural_complexity] src/app/actions.rs
    summary: Structural complexity: nesting depth 9
    question: Can the nesting be reduced with early returns, guard clauses, or extraction into helper functions? Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned, structural
    evidence: File size: 1173 lines
  - [structural_complexity] src/app/events.rs
    summary: Structural complexity: nesting depth 12
    question: Can the nesting be reduced with early returns, guard clauses, or extraction into helper functions? Is this file truly dead, or is it used via a non-import mechanism (dynamic import, CLI entry point, plugin)?
    evidence: Flagged by: orphaned, structural
    evidence: File size: 567 lines
  - (+1 more concern signals)

Task requirements:
1. Read the blind packet's `system_prompt` — it contains scoring rules and calibration.
2. Start from the seed files, then freely explore the repository to build your understanding.
3. Keep issues and scoring scoped to this batch's dimension.
4. Respect scope controls: do not include files/directories marked by `exclude`, `suppress`, or non-production zone overrides.
5. Return 0-10 issues for this batch (empty array allowed).
6. For design_coherence, use evidence from `holistic_context.scan_evidence.signal_density` — files where multiple mechanical detectors fired. Investigate what design change would address multiple signals simultaneously. Check `scan_evidence.complexity_hotspots` for files with high responsibility cluster counts.
7. Workflow integrity checks: when reviewing orchestration/queue/review flows,
8. xplicitly look for loop-prone patterns and blind spots:
9. - repeated stale/reopen churn without clear exit criteria or gating,
10. - packet/batch data being generated but dropped before prompt execution,
11. - ranking/triage logic that can starve target-improving work,
12. - reruns happening before existing open review work is drained.
13. If found, propose concrete guardrails and where to implement them.
14. Do not edit repository files.
15. Return ONLY valid JSON, no markdown fences.

Scope enums:
- impact_scope: "local" | "module" | "subsystem" | "codebase"
- fix_scope: "single_edit" | "multi_file_refactor" | "architectural_change"

Output schema:
{
  "batch": "design_coherence",
  "batch_index": 11,
  "assessments": {"<dimension>": <0-100 with one decimal place>},
  "dimension_notes": {
    "<dimension>": {
      "evidence": ["specific code observations"],
      "impact_scope": "local|module|subsystem|codebase",
      "fix_scope": "single_edit|multi_file_refactor|architectural_change",
      "confidence": "high|medium|low",
      "issues_preventing_higher_score": "required when score >85.0",
      "sub_axes": {"abstraction_leverage": 0-100, "indirection_cost": 0-100, "interface_honesty": 0-100, "delegation_density": 0-100, "definition_directness": 0-100, "type_discipline": 0-100}  // required for abstraction_fitness when evidence supports it; all one decimal place
    }
  },
  "issues": [{
    "dimension": "<dimension>",
    "identifier": "short_id",
    "summary": "one-line defect summary",
    "related_files": ["relative/path.py"],
    "evidence": ["specific code observation"],
    "suggestion": "concrete fix recommendation",
    "confidence": "high|medium|low",
    "impact_scope": "local|module|subsystem|codebase",
    "fix_scope": "single_edit|multi_file_refactor|architectural_change",
    "root_cause_cluster": "optional_cluster_name_when_supported_by_history"
  }],
  "retrospective": {
    "root_causes": ["optional: concise root-cause hypotheses"],
    "likely_symptoms": ["optional: identifiers that look symptom-level"],
    "possible_false_positives": ["optional: prior concept keys likely mis-scoped"]
  }
}
