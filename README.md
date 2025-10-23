# Compiler for Lisp

This is a demo project, and still in development.

This implementation generates machine code directly, instead of generating text assembly.
I only choose this to learn more about compilers and machine code.

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
