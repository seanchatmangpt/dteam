//! Declarative macros for COG8 graph construction.

/// Defines a COG8 graph (nodes and edges) using a clean DSL.
///
/// This macro returns a tuple containing two fixed-size arrays:
/// `([Cog8Row; N], [Cog8Edge; M])`.
///
/// # Example
/// ```
/// use ccog::cog8_graph;
/// use ccog::runtime::cog8::{Powl8Op, Instinct};
/// use ccog::ids::{PackId, FieldId};
///
/// let (nodes, edges) = cog8_graph! {
///     nodes: [
///         node(0) {
///             pack_id: PackId(1),
///             response: Instinct::Settle,
///             priority: 10,
///         },
///         node(1) {
///             pack_id: PackId(1),
///             response: Instinct::Retrieve,
///         },
///     ],
///     edges: [
///         node(0) -> node(1) via Choice {
///             op: Powl8Op::Act,
///             effect_mask: 0x1,
///         },
///     ]
/// };
///
/// assert_eq!(nodes.len(), 2);
/// assert_eq!(edges.len(), 1);
/// assert_eq!(edges[0].kind, ccog::runtime::cog8::EdgeKind::Choice);
/// ```
#[macro_export]
macro_rules! cog8_graph {
    (
        nodes: [ $(node($nid:expr) { $($nfield:ident : $nval:expr),* $(,)? }),* $(,)? ],
        edges: [ $(node($from:expr) -> node($to:expr) via $kind:ident { $($efield:ident : $eval:expr),* $(,)? }),* $(,)? ]
    ) => {{
        let nodes = [
            $(
                {
                    #[allow(unused_mut)]
                    let mut node = $crate::runtime::cog8::Cog8Row::default();
                    $(
                        node.$nfield = $nval;
                    )*
                    node
                },
            )*
        ];
        let edges = [
            $(
                {
                    #[allow(unused_mut)]
                    let mut instr = $crate::runtime::cog8::Powl8Instr::default();
                    instr.node_id = $crate::ids::NodeId($to as u16);
                    $(
                        instr.$efield = $eval;
                    )*
                    $crate::runtime::cog8::Cog8Edge {
                        from: $crate::ids::NodeId($from as u16),
                        to: $crate::ids::NodeId($to as u16),
                        kind: $crate::runtime::cog8::EdgeKind::$kind,
                        instr,
                    }
                },
            )*
        ];
        (nodes, edges)
    }};
}
