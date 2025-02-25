use oxc_ast::{
    ast::{
        BindingPatternKind, ExportDefaultDeclarationKind, Expression, ModuleDeclaration, Statement,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::get_declaration_of_variable, context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-async-client-component): Prevent client components from being async functions.")]
#[diagnostic(
    severity(warning),
    help("See: https://nextjs.org/docs/messages/no-async-client-component")
)]
struct NoAsyncClientComponentDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAsyncClientComponent;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoAsyncClientComponent,
    correctness
);

impl Rule for NoAsyncClientComponent {
    fn run_once(&self, ctx: &LintContext) {
        let Some(root) = ctx.nodes().iter().next() else { return };
        let AstKind::Program(program) = root.kind() else { return };

        if program
            .directives
            .iter()
            .any(|directive| directive.expression.value.as_str() == "use client")
        {
            for node in &program.body {
                let Statement::ModuleDeclaration(mod_decl) = &node else {
                    continue;
                };
                let ModuleDeclaration::ExportDefaultDeclaration(export_default_decl) = &**mod_decl
                else {
                    continue;
                };

                // export default async function MyComponent() {...}
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) =
                    &export_default_decl.declaration
                {
                    if func_decl.r#async
                        && func_decl
                            .id
                            .as_ref()
                            .is_some_and(|v| v.name.chars().next().unwrap().is_uppercase())
                    {
                        ctx.diagnostic(NoAsyncClientComponentDiagnostic(
                            func_decl.id.as_ref().unwrap().span,
                        ));
                    }
                    continue;
                }

                // async function MyComponent() {...}; export default MyComponent;
                if let ExportDefaultDeclarationKind::Expression(Expression::Identifier(
                    export_default_id,
                )) = &export_default_decl.declaration
                {
                    let Some(decl) = get_declaration_of_variable(export_default_id, ctx) else {
                        continue;
                    };

                    if let AstKind::Function(func) = decl.kind() {
                        if func.r#async
                            && func
                                .id
                                .as_ref()
                                // `func.id.name` MUST be > 0 chars
                                .is_some_and(|v| v.name.chars().next().unwrap().is_uppercase())
                        {
                            ctx.diagnostic(NoAsyncClientComponentDiagnostic(
                                func.id.as_ref().unwrap().span,
                            ));
                        }
                    }

                    if let AstKind::VariableDeclarator(var_declarator) = decl.kind() {
                        if let BindingPatternKind::BindingIdentifier(binding_ident) =
                            &var_declarator.id.kind
                        {
                            // `binding_ident.name` MUST be > 0 chars
                            if binding_ident.name.chars().next().unwrap().is_uppercase() {
                                if let Some(Expression::ArrowExpression(arrow_expr)) =
                                    &var_declarator.init
                                {
                                    if arrow_expr.r#async {
                                        ctx.diagnostic(NoAsyncClientComponentDiagnostic(
                                            binding_ident.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
			    export default async function MyComponent() {
			      return <></>
			    }
			    ",
        r#"
			    "use client"
			
			    export default async function myFunction() {
			      return ''
			    }
			    "#,
        r"
			    async function MyComponent() {
			      return <></>
			    }
			
			    export default MyComponent
			    ",
        r#"
			    "use client"
			
			    async function myFunction() {
			      return ''
			    }
			
			    export default myFunction
			    "#,
        r#"
			    "use client"
			
			    const myFunction = () => {
			      return ''
			    }
			
			    export default myFunction
			    "#,
    ];

    let fail = vec![
        r#"
			      "use client"
			
			      export default async function MyComponent() {
			        return <></>
			      }
			      "#,
        r#"
			      "use client"
			
			      export default async function MyFunction() {
			        return ''
			      }
			      "#,
        r#"
			      "use client"
			
			      async function MyComponent() {
			        return <></>
			      }
			
			      export default MyComponent
			      "#,
        r#"
			      "use client"
			
			      async function MyFunction() {
			        return ''
			      }
			
			      export default MyFunction
			      "#,
        r#"
			      "use client"
			
			      const MyFunction = async () => {
			        return '123'
			      }
			
			      export default MyFunction
			      "#,
    ];

    Tester::new_without_config(NoAsyncClientComponent::NAME, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
