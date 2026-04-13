# Story Dossier Cascade

This sample project builds `story-dossier.md` one top-level worksheet section at a time.

## How it works

- `braindump.md` is used only for `section_1_required_data_layer`
- `story-dossier-worksheet-template.sections.md` is the top-level wrapped worksheet shell
- `story-dossier.md` is the appended output document
- each run appends exactly one wrapped section to `story-dossier.md`

## Section Order

1. `section_1_required_data_layer`
2. `section_2_story_concept`
3. `section_3_protagonist_operating_systems`
4. `section_4_supporting_cast`
5. `section_5_story_world`
6. `section_6_narrative_physics_engine_axes_and_vectors`
7. `section_7_narrative_physics_engine_thresholds_and_entropy`
8. `section_8_writing_style_rules`
9. `section_9_genre_lens`
10. `section_10_story_summary`
11. `section_11_structure_breakdown`
12. `section_12_chapter_outlines_setup`
13. `section_13_chapter_outlines_rising_action`
14. `section_14_chapter_outlines_complications`
15. `section_15_chapter_outlines_climax`
16. `section_16_chapter_outlines_resolution`
17. `section_17_continuity_check`

## Important Notes

- This project uses `append_document`, so rerunning a section appends a duplicate section unless you clear `story-dossier.md` first.
- The worksheet template is normalized only at the `## section_*` level so that internal `###` and `####` structure remains part of the section content.
- Later sections use the completed dossier so far as upstream context instead of rereading the original braindump.
