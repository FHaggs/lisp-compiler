# JIT Runtime for Lisp

This is a project to learn about compilers and machine code by building a JIT (Just-In-Time) runtime for a Lisp-like language. This is just for fun, you should not take this ideas into production.

Plans:
- [x] Compile integers
- [x] Compile other immediate constants (booleans, ASCII characters, the empty list)
- [ ] Unary expr
- [ ] Binary expr
- [ ] Parser
- [ ] Local variables (let keyword)
- [ ] Conditionals
- [ ] Heap alloc (Cons list, symbols, strings)
- [ ] Compile procedure calls (labels, code, and labelcall)
- [ ] Compile closures
- [ ] Add tail-call optimization
- [ ] Compile complex constants (quote)
- [ ] Compile variable assignment (set!)
- [ ] Add macro expander
- [ ] Foreign function calls
