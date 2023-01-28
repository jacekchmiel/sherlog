# Sherlog

The log detective.

...starts as simple terminal log viewer.

When it proves useful, maybe will get other front-ends - e.g. vscode/firefox
plugin or full-blown standalone GUI.

## Roadmap

### MVP

 - [x] less-like interface with basic scrolling
 - [x] highlighting with regex
 - [x] filtering with regex (single filter)
 - [ ] filtering with regex (multiple negative and positive filters)
   - [ ] active filter list with ability to add remove
   - [ ] disabling filters on the list (without removal to be able to enable
   them again later)
 - [ ] higlighting with regex
   - [ ] highlighting rules on filter list
 - [ ] saving view state automatically
   (stores filtering/higlighting state, restores when file is opened again)

### Ideas for future far away

- Parsing - converting text to structured log
- Structured log support with filtering/highlighting rules on fields
- Timestamp normalization allowing easier navigation and filtering (e.g. show
  me all the logs five minutes into the future from this point)
- extract sherlog-core and creating non-terminal frontends
- create form of mini-language to control sherlog pipeline (parsing.
  filtering, highlighting)
