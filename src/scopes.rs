/*!
This module contains all the scopes.
The cpp code requires that certain methods may only be called in certain scopes.

As soon as you enter a deeper scope you are not allowed to call methods from the other scope inside the inner one.
This is not enforced by the typesystem as I don't want to type so much and the error messages in the asserts in the cpp code will make errors easier to see.

This structure reduces the ammount of runtime errors while also making it eaiser to discover available methods.
*/

use crate::{
    sys, AttributeId, EditorContext, Hoverable, InputPinId, Link, LinkId, NodeId, OutputPinId,
    PinId, PinShape,
};

/// entry point
///
/// BeginNodeEditor
/// ...
/// EndNodeEditor
pub fn editor<F: FnOnce(ScopeEditor)>(context: &EditorContext, f: F) -> ScopeNone {
    context.set_as_current_editor();

    unsafe { sys::imnodes_BeginNodeEditor() };
    f(ScopeEditor {});
    unsafe { sys::imnodes_EndNodeEditor() };
    ScopeNone {}
}

/// Original Scopes turned into compile time checks:
/// Scope_None = 1,
#[derive(Debug)]
pub struct ScopeNone {}
impl ScopeNone {
    /// check the position of the mosue
    pub fn is_hovered(&self, id: impl Hoverable) -> bool {
        id.is_hovered(&self)
    }

    /// IsLinkStarted
    pub fn link_started_at<T: Into<PinId>>(&self, id: T) -> bool {
        let id: PinId = id.into();
        id.is_start_of_link(self)
    }

    /// IsLinkDropped
    pub fn link_dropped_from<T: Into<PinId>>(&self, id: T, including_detached_links: bool) -> bool {
        let id: PinId = id.into();
        id.dropped_link(including_detached_links, self)
    }

    /// NumSelectedNodes
    /// selected_nodes builds on top of this
    pub fn num_selected_nodes(&self) -> u32 {
        let num = unsafe { sys::imnodes_NumSelectedNodes() };
        assert!(num > 0);
        num as u32
    }

    /// NumSelectedLinks
    /// selected_links builds on top of this
    pub fn num_selected_links(&self) -> u32 {
        let num = unsafe { sys::imnodes_NumSelectedLinks() };
        assert!(num > 0);
        num as u32
    }

    /// GetSelectedNodes
    pub fn selected_nodes(&self) -> Vec<NodeId> {
        let nr_nodes = self.num_selected_nodes() as usize;
        let mut nodes = vec![NodeId { id: 0 }; nr_nodes];
        unsafe { sys::imnodes_GetSelectedNodes(nodes.as_mut_ptr() as _) };
        nodes
    }

    /// GetSelectedLinks
    pub fn selected_links(&self) -> Vec<LinkId> {
        let nr_links = self.num_selected_links() as usize;
        let mut links = vec![LinkId { id: 0 }; nr_links];
        unsafe { sys::imnodes_GetSelectedLinks(links.as_mut_ptr() as _) };
        links
    }

    /// IsLinkCreated
    pub fn links_created(&self) -> Option<Link> {
        let mut started_at_node_id: i32 = -1;
        let mut started_at_attribute_id: i32 = -1;
        let mut ended_at_node_id: i32 = -1;
        let mut ended_at_attribute_id: i32 = -1;
        let mut created_from_snap: bool = true;

        let is_created = unsafe {
            sys::imnodes_IsLinkCreatedIntPtr(
                &mut started_at_node_id as _,
                &mut started_at_attribute_id as _,
                &mut ended_at_node_id as _,
                &mut ended_at_attribute_id as _,
                &mut created_from_snap as *mut bool,
            )
        };

        // let is_created = unsafe {
        //     sys::imnodes_IsLinkCreatedBoolPtr(
        //         &mut started_at_attribute_id as _,
        //         &mut ended_at_attribute_id as _,
        //         &mut created_from_snap as *mut bool,
        //     )
        // };

        if is_created {
            Some(Link {
                start_node: NodeId {
                    id: started_at_node_id,
                },
                end_node: NodeId {
                    id: ended_at_node_id,
                },
                start_pin: OutputPinId {
                    id: started_at_attribute_id,
                },
                end_pin: InputPinId {
                    id: ended_at_attribute_id,
                },
                craeated_from_snap: created_from_snap,
            })
        } else {
            None
        }
    }

    /// IsLinkDestroyed
    pub fn get_dropped_link(&self) -> Option<LinkId> {
        let mut id: i32 = -1;
        if unsafe { sys::imnodes_IsLinkDestroyed(&mut id as _) } {
            Some(LinkId { id })
        } else {
            None
        }
    }

    /// IsPinHovered
    pub fn get_hovered_pin(&self) -> Option<PinId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsPinHovered(&mut id as _) };
        if ok {
            Some(PinId { id })
        } else {
            None
        }
    }

    /// IsLinkHovered
    pub fn get_hovered_link(&self) -> Option<LinkId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsLinkHovered(&mut id as _) };
        if ok {
            Some(LinkId { id })
        } else {
            None
        }
    }

    /// IsAnyAttributeActive
    pub fn get_active_attribute(&self) -> Option<AttributeId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsAnyAttributeActive(&mut id as _) };
        if ok {
            Some(AttributeId { id })
        } else {
            None
        }
    }

    /// IsLinkStarted
    pub fn from_where_link_started(&self) -> Option<PinId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsLinkStarted(&mut id as _) };
        if ok {
            Some(PinId { id })
        } else {
            None
        }
    }

    /// IsLinkDropped
    pub fn from_where_link_dropped(&self, including_detached_links: bool) -> Option<PinId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsLinkDropped(&mut id as _, including_detached_links) };
        if ok {
            Some(PinId { id })
        } else {
            None
        }
    }
}

/// Scope_Editor = 1 << 1,
#[derive(Debug)]
pub struct ScopeEditor {}
impl ScopeEditor {
    /// BeginNode
    /// ...
    /// EndNode
    pub fn node<F: FnOnce(ScopeNode)>(&self, id: NodeId, f: F) {
        unsafe { sys::imnodes_BeginNode(id.into()) }

        f(ScopeNode {});
        unsafe { sys::imnodes_EndNode() };
    }

    /// Link
    pub fn add_link(&self, id: LinkId, input: InputPinId, output: OutputPinId) {
        unsafe { sys::imnodes_Link(id.into(), input.into(), output.into()) }
    }

    /// IsAnyAttributeActive
    pub fn get_active_attribute(&self) -> Option<AttributeId> {
        let mut id: i32 = -1;
        let ok = unsafe { sys::imnodes_IsAnyAttributeActive(&mut id as _) };
        if ok {
            Some(AttributeId { id })
        } else {
            None
        }
    }

    /// IsEditorHovered
    pub fn is_hovered(&self) -> bool {
        unsafe { sys::imnodes_IsEditorHovered() }
    }
}

/// Scope_Node = 1 << 2,
#[derive(Debug)]
pub struct ScopeNode {}
impl ScopeNode {
    /// BeginNodeTitleBar
    /// ....
    /// EndNodeTitleBar
    pub fn add_titlebar<F: FnOnce()>(&self, f: F) {
        unsafe { sys::imnodes_BeginNodeTitleBar() }
        f();
        unsafe { sys::imnodes_EndNodeTitleBar() }
    }

    /// BeginInputAttribute
    /// ...
    /// EndInputAttribute
    pub fn add_input<F: FnOnce()>(&self, id: InputPinId, shape: PinShape, f: F) {
        unsafe { sys::imnodes_BeginInputAttribute(id.into(), shape as i32) };
        f();
        unsafe { sys::imnodes_EndInputAttribute() };
    }

    /// BeginOutputAttribute
    /// ...
    /// EndOutputAttribute
    pub fn add_output<F: FnOnce()>(&self, id: OutputPinId, shape: PinShape, f: F) {
        unsafe { sys::imnodes_BeginOutputAttribute(id.into(), shape as i32) };
        f();
        unsafe { sys::imnodes_EndOutputAttribute() };
    }

    /// BeginStaticAttribute
    /// ...
    /// EndStaticAttribute
    pub fn attribute<F: FnOnce()>(&self, id: AttributeId, f: F) {
        unsafe { sys::imnodes_BeginStaticAttribute(id.into()) };
        f();
        unsafe { sys::imnodes_EndStaticAttribute() };
    }
}