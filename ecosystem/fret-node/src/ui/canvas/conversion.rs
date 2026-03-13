//! Typed conversion helpers for connection workflows.

use crate::core::{CanvasPoint, EdgeId, Graph, PortDirection, PortId};
use crate::rules::{ConnectPlan, InsertNodeTemplate, plan_connect_by_inserting_node};
use crate::ui::canvas::geometry::node_size_default_px;
use crate::ui::presenter::{InsertNodeCandidate, NodeGraphPresenter};
use crate::ui::style::NodeGraphStyle;

mod conversion_candidates;
mod conversion_plan;

pub(crate) use conversion_candidates::{build_picker_candidates, is_convertible};
pub(crate) use conversion_plan::{plan_insert_conversion, try_auto_insert_conversion};
