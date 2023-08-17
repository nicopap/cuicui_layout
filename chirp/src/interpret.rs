use std::marker::PhantomData;

use bevy::{
    prelude::{BuildChildren, ChildBuilder},
    utils::HashMap,
};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use thiserror::Error;

use crate::{parse, ParseDsl};

type InterpResult = Result<(), InterpError>;

/// An error occuring when adding a [`crate::Chirp`] to the world.
#[allow(missing_docs)] // Already documented by error message.
#[derive(Debug, Error)]
pub enum InterpError {
    // TODO(err): Integrate parse spans for nice error reporting.
    #[error(
        "KDL method is malformed, it should have a name, either as \
        parameter name or as KDL argument string"
    )]
    NoName,
    #[error(
        "The KDL method had a non-string arg. You should only use strings in \
        argument position."
    )]
    BadArg,
    #[error("'code' should have exactly one string argument, none were given")]
    BadCode,
    #[error("'code' should be a rust identifier, found '{0}'")]
    CodeNonIdent(String),
    #[error("Didn't find the code handle '{0}' in provided code handles")]
    CodeNotPresent(String),
    #[error("leaf nodes should have at least one argument to be passed as as value")]
    LeafNoArgs,
    #[error(
        "leaf nodes expect values to have a str representation. You passed a \
        custom-built Kdl document without specifying a leaf node representation"
    )]
    LeafBadKdl,
    #[error(transparent)]
    DslError(#[from] anyhow::Error),
}

fn kdl_args(kdl: &KdlEntry) -> Result<&str, InterpError> {
    use KdlValue::{RawString, String};
    match kdl.value() {
        _ if kdl.name().is_none() => Ok(""),
        RawString(value) | String(value) => Ok(value),
        _ => Err(InterpError::BadArg),
    }
}
fn kdl_name(kdl: &KdlEntry) -> Result<&str, InterpError> {
    use KdlValue::{RawString, String};
    match (kdl.name(), kdl.value()) {
        (Some(name), _) => name.repr().ok_or(InterpError::NoName),
        (None, RawString(name) | String(name)) => Ok(name),
        _ => Err(InterpError::NoName),
    }
}
// TODO(clean) TODO(feat): Consider replacing this with a trait that takes
// `handle(&str, &mut ChildBuilder)`, so that it is concievable of not relying
// on dynamic dispatch.
/// Registry of functions used in `code` block in [`crate::Chirp`]s.
pub type Handles<'h> = HashMap<String, &'h dyn Fn(&mut ChildBuilder)>;

pub(super) struct DslInterpret<'h, 'h2, 'b, D> {
    _dsl: PhantomData<D>,
    handles: &'h Handles<'h2>,
    load_context: PhantomData<&'b ()>,
}
impl<'h, 'h2, 'b, D: ParseDsl> DslInterpret<'h, 'h2, 'b, D> {
    pub(super) fn new(handles: &'h Handles<'h2>) -> Self {
        Self {
            _dsl: PhantomData,
            load_context: PhantomData,
            handles,
        }
    }
    fn statement(&self, kdl: &KdlNode, cmds: &mut ChildBuilder) -> InterpResult {
        let mut dsl_bundle = D::default();
        if kdl.name().value() == "code" {
            let Some(handle) = kdl.entries().first() else {
                return Err(InterpError::BadCode);
            };
            let Some(handle) = handle.value().as_string() else {
                return Err(InterpError::CodeNonIdent(handle.to_string()));
            };
            let Some(to_run) = self.handles.get(handle) else {
                return Err(InterpError::CodeNotPresent(handle.to_owned()));
            };
            to_run(cmds);
            return Ok(());
        }
        let mut cmds = cmds.spawn_empty();
        let mut entries = kdl.entries();
        // Skip first entry if leaf-node (the first entry should be an argument
        // to the leaf-node method)
        if kdl.children().is_none() {
            entries = &entries[1..];
        }
        for entry in entries {
            dsl_bundle.method(parse::InterpretMethodCtx {
                name: kdl_name(entry)?,
                args: kdl_args(entry)?,
            })?;
        }
        if let Some(document) = kdl.children() {
            // Apply the parent node method.
            if kdl.name().value() != "spawn" {
                dsl_bundle
                    .method(parse::InterpretMethodCtx { name: kdl.name().value(), args: "" })?;
            }
            dsl_bundle.insert(&mut cmds);
            let mut err = Ok(());
            cmds.with_children(|cmds| {
                let children = Self {
                    _dsl: PhantomData,
                    handles: self.handles,
                    load_context: PhantomData,
                };
                err = children.statements(document, cmds);
            });
            err
        } else {
            if kdl.name().value() != "spawn" {
                let Some(leaf_arg) = kdl.entries().first() else {
                    return Err(InterpError::LeafNoArgs);
                };
                let Some(leaf_arg) = leaf_arg.value_repr() else {
                    return Err(InterpError::LeafBadKdl);
                };
                dsl_bundle.leaf_node(parse::InterpretLeafCtx {
                    name: kdl.name().value(),
                    leaf_arg,
                    cmds: &mut cmds,
                })?;
            }
            dsl_bundle.insert(&mut cmds);
            Ok(())
        }
    }
    pub(super) fn statements(self, kdl: &KdlDocument, cmds: &mut ChildBuilder) -> InterpResult {
        for node in kdl.nodes() {
            self.statement(node, cmds)?;
        }
        Ok(())
    }
}
