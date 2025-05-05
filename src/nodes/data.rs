use crate::arenal::Idx;
use crate::ast::nodes::ExpressionAst;
use crate::err;
use crate::nodes::item::ItemIdx;
use crate::nodes::types::{PropsBox, StateBox};
use crate::result::ViuiResult;
use crate::types::Rect;
use std::any::type_name;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub struct NodeData {
    pub tag: String,
    pub kind_index: usize,
    pub layout: LayoutInfo,
    pub state: StateBox,
    pub props: PropsBox,
    pub children: Vec<ItemIdx>,
    pub prop_expressions: Vec<PropExpression>,
    pub event_mappings: HashMap<String, ExpressionAst>,
}

pub type NodeIdx = Idx<NodeData>;

impl Debug for NodeData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.tag)
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayoutInfo {
    pub bounds: Rect,
}

#[derive(Clone)]
pub struct PropExpression {
    pub field_name: String,
    pub expression: ExpressionAst,
}

impl NodeData {
    pub fn cast_state_mut_and_props<S: 'static, P: 'static>(&mut self) -> ViuiResult<(&mut S, &P)> {
        let state = self
            .state
            .as_any_mut()
            .downcast_mut::<S>()
            .ok_or_else(|| err!("Could not cast state to actual type {}", type_name::<S>()))?;
        let props = self
            .props
            .as_any()
            .downcast_ref::<P>()
            .ok_or_else(|| err!("Could not cast props to actual type {}", type_name::<P>()))?;
        Ok((state, props))
    }

    pub fn cast_state_and_props<S: 'static, P: 'static>(&self) -> ViuiResult<(&S, &P)> {
        let state = self
            .state
            .as_any()
            .downcast_ref::<S>()
            .ok_or_else(|| err!("Could not cast state to actual type {}", type_name::<S>()))?;
        let props = self
            .props
            .as_any()
            .downcast_ref::<P>()
            .ok_or_else(|| err!("Could not cast props to actual type {}", type_name::<P>()))?;
        Ok((state, props))
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.layout.bounds = bounds;
    }

    pub fn bounds(&self) -> &Rect {
        &self.layout.bounds
    }

    pub fn kind_index(&self) -> usize {
        self.kind_index
    }
}
