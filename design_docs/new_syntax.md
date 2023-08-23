# New syntax

[Cart's proposal][cpro] gave me ideas:

* Remove the "leaf node" syntax (using arbitrary methods where `spawn` and `code`
  are valid) It's confusing to have an two ways of doing the same thing.
* In place of this, use the "leaf node" as the entity name
* Remove the name literal method syntax
* "Reserve" rust keywords for future compatibility.

[cpro]: https://github.com/bevyengine/bevy/discussions/9538

## Grammar

```ungrammar
Method
   = 'ident'                  // bare method
   | 'ident' ':' 'expr'       // property method
   | 'ident' '(' ExprList ')' // argument method

StatementHead
   = 'code'    'ident'
   | 'rust_kw' MethodList
   | 'entity'  MethodList
   | 'ident'   MethodList
   | StringLit MethodList

StatementTail = ';' | '{' Statement* '}'
Statement     = StatementHead statementTail

MethodList = Method (',' Method)* ','?
ExprList   = 'expr' (',' 'expr')* ','?
StringLit  = 'string'
```

* `'rust_kw'`: Any existing rust keyword such as `for`, `in`, `let`, `fn` etc.
* When `StatementHead` is an arbitrary identifier or a `StringLit`, it is used
  as the entity's name.