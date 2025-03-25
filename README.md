# `jrnl`
A simple program to list the day's activities.

### TODO / Roadmap
- [ ] Folder to save all jrnls.
- [ ] Folder structure:
  ``` 
  .jrnl
  |
  '- year1
  |  '- year1_month1.md
  |  '- year1_month2.md
  '- year2
  |  '- year2_month1.md
  |  '- year2_month2.md
  ```
- [ ] typing `jrnl` should open the current month's file and enter today's date and day. If already entered, it should not duplicate itself.
- [ ] Data to be entered as unordered lists.
- [ ] Tags should be allowed as:
  ```md
  # date (day)
  - [Tag1] Did this stuff.
  - Did some other stuff.
  - [Tag2] Also some stuff.
  ```
- [ ] All files throughout all years must be searchable both for tags and words/phrases.
