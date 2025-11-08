# todo

- [ ] make lang module
- [ ] design & spec grammar
- [ ] start repl?
- [ ] impl tokenizer (this does come before parser, right?)
- [ ] start parser
  - will do as shift reduce parser (for lr(1)? grammar)
    - read more [from lecture notes here](https://www.cs.uaf.edu/~chappell/class/2025_spr/cs331/lect/cs331-20250217-shiftred.pdf) ~~
    - designed as a stack (call stack?) pulling input tokens from source &
      handling Actions & States as enums w/ possible related values
- [ ] design & spec ast
- [ ] build ast -> llvm ir repr?
