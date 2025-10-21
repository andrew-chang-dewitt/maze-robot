# todo

- [/] refactor maze & related behaviours
- [ ] refactor robot behaviours
  - [ ] abstract to traits
  - [ ] create TextRobot implementation of those traits (leaves ready for
        ArduinoRobot impl)
- [ ] refactor solution algo to have no knowledge of maze beyond what it
      discovers (e.g. just gets a start position and an opaque Robot obj w/ Peek &
      Move behaviours that it can use to build it's own internal maze repr)
- [ ] impl Tokens
- [ ] impl TextParser
- [ ] impl translation? (compilation?) layer to target lang (rust? arduino
      machine code?)
