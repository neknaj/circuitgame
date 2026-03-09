use crate::compiler;
use crate::compiler::types::{
    CompiledGateInput, Component, Graphical, ImgSize, IntermediateProducts,
};
use crate::test;
use crate::test::types::TestProducts;
use crate::transpiler;
use crate::vm;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, RichText, Sense, Stroke, TextStyle};
use std::collections::HashMap;
use std::ops::Range;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
#[cfg(not(target_arch = "wasm32"))]
use image::{Rgba, RgbaImage};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const SAMPLE_SOURCE: &str = include_str!("../../spec/sample.ncg");
const CANVAS_ID: &str = "circuitgame-egui";
const CIRCUIT_GRAPH_HORIZONTAL_PADDING: f32 = 24.0;
const CIRCUIT_GRAPH_VERTICAL_PADDING_TOP: f32 = 28.0;
const CIRCUIT_GRAPH_VERTICAL_PADDING_BOTTOM: f32 = 36.0;
const CIRCUIT_GRAPH_EXPORT_MIN_SIZE: u32 = 256;
const CIRCUIT_GRAPH_EXPORT_MAX_SIZE: u32 = 8192;

pub struct CircuitGameApp {
    source: String,
    auto_compile: bool,
    compile_state: CompileState,
    selected_module: Option<String>,
    simulation: Option<SimulationState>,
    auto_run: bool,
    auto_run_delay_ms: u64,
    last_auto_step_at: f64,
    logic_window: usize,
    source_panel_width: f32,
    simulation_panel_width: f32,
    transpile_panel_height: f32,
    circuit_graph_height: f32,
    show_source_panel: bool,
    show_transpile_panel: bool,
    show_simulation_controls: bool,
    show_simulation_details: bool,
    show_diagnostics_panel: bool,
    show_circuit_graph: bool,
    circuit_graph_export_width: u32,
    circuit_graph_export_height: u32,
    circuit_graph_export_status: Option<String>,
    socket_status: Option<String>,
}

struct CompileState {
    products: IntermediateProducts,
    tests: TestProducts,
    transpiled_ts: String,
}

struct SimulationState {
    module_name: String,
    vm: vm::types::Module,
    inputs: Vec<bool>,
    outputs: Vec<bool>,
    input_labels: Vec<String>,
    output_labels: Vec<String>,
    wave_labels: Vec<String>,
    wave_data: Vec<Vec<bool>>,
    graphical: Option<Graphical>,
    circuit_gate_layout: CircuitGraphGateLayout,
}

struct CircuitGraphGroup {
    id: usize,
    label: String,
    gate_range: Range<usize>,
    children: Vec<CircuitGraphGroup>,
}

#[derive(Clone)]
struct CircuitGraphGroupLayout {
    id: usize,
    label: String,
    depth: usize,
    rect: egui::Rect,
    children: Vec<CircuitGraphGroupLayout>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CircuitGraphLabelAnchor {
    InsideTopLeft,
    InsideTopRight,
    AboveLeft,
    AboveRight,
    BelowLeft,
    BelowRight,
    OutsideLeft,
    OutsideRight,
}

#[derive(Clone)]
struct CircuitGraphLabelPlacement {
    label: String,
    rect: egui::Rect,
    anchor: CircuitGraphLabelAnchor,
}

#[derive(Clone, Copy, Debug, Default)]
struct CircuitGraphLayoutEvaluation {
    total: f32,
    label_overlap: f32,
    label_proximity: f32,
    gate_overlap: f32,
    gate_proximity: f32,
    out_of_bounds: f32,
    anchor_cost: f32,
}

#[derive(Clone)]
struct CircuitGraphGateLayout {
    gate_levels: Vec<usize>,
    gate_ranks: Vec<usize>,
    level_counts: Vec<usize>,
    max_level: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CircuitGraphNodeRef {
    Input(usize),
    Gate(usize),
    Output(usize),
}

#[derive(Clone, Copy)]
struct CircuitGraphEdgeRef {
    source: CircuitGraphNodeRef,
    target: CircuitGraphNodeRef,
}

#[derive(Clone, Copy, Debug, Default)]
struct CircuitGraphWireEvaluation {
    total: f32,
    crossings: f32,
    same_level_penalty: f32,
    backward_penalty: f32,
}

struct CircuitGraphRenderScene {
    rect: egui::Rect,
    input_positions: Vec<egui::Pos2>,
    gate_positions: Vec<egui::Pos2>,
    output_positions: Vec<egui::Pos2>,
    gate_states: Vec<bool>,
    group_layout: Option<CircuitGraphGroupLayout>,
    label_layouts: HashMap<usize, CircuitGraphLabelPlacement>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct SocketInbox {
    latest_source: Option<String>,
    latest_status: Option<String>,
}

#[cfg(target_arch = "wasm32")]
struct SocketRuntime {
    _socket: web_sys::WebSocket,
    _on_open: Closure<dyn FnMut(web_sys::Event)>,
    _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _on_error: Closure<dyn FnMut(web_sys::Event)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SOCKET_INBOX: RefCell<SocketInbox> = RefCell::new(SocketInbox::default());
    static SOCKET_RUNTIME: RefCell<Option<SocketRuntime>> = RefCell::new(None);
    static SOCKET_RECONNECT: RefCell<Option<Closure<dyn FnMut()>>> = RefCell::new(None);
}

impl CircuitGameApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = Self {
            source: SAMPLE_SOURCE.to_string(),
            auto_compile: true,
            compile_state: CompileState::empty(),
            selected_module: None,
            simulation: None,
            auto_run: false,
            auto_run_delay_ms: 120,
            last_auto_step_at: 0.0,
            logic_window: 64,
            source_panel_width: 460.0,
            simulation_panel_width: 640.0,
            transpile_panel_height: 220.0,
            circuit_graph_height: 320.0,
            show_source_panel: true,
            show_transpile_panel: true,
            show_simulation_controls: true,
            show_simulation_details: true,
            show_diagnostics_panel: true,
            show_circuit_graph: true,
            circuit_graph_export_width: 1920,
            circuit_graph_export_height: 1080,
            circuit_graph_export_status: None,
            socket_status: None,
        };
        app.init_socket_bridge(&cc.egui_ctx);
        app.recompile();
        app
    }

    fn recompile(&mut self) {
        let products = compiler::intermediate_products(&self.source);
        let tests = if products.errors.is_empty() {
            test::test(products.clone())
        } else {
            empty_test_products()
        };
        let transpiled_ts = transpile_all_modules(&products);
        self.compile_state = CompileState {
            products,
            tests,
            transpiled_ts,
        };
        self.sync_selected_module();
        self.rebuild_simulation();
    }

    fn sync_selected_module(&mut self) {
        let module_names = self
            .compile_state
            .products
            .module_type_list
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>();
        if module_names.is_empty() {
            self.selected_module = None;
            return;
        }
        let selected_is_valid = self
            .selected_module
            .as_deref()
            .map(|name| module_names.iter().any(|candidate| *candidate == name))
            .unwrap_or(false);
        if !selected_is_valid {
            self.selected_module = self
                .compile_state
                .products
                .module_dependency_sorted
                .first()
                .cloned()
                .or_else(|| {
                    self.compile_state
                        .products
                        .module_type_list
                        .first()
                        .map(|module| module.name.clone())
                });
        }
    }

    fn rebuild_simulation(&mut self) {
        let Some(module_name) = self.selected_module.clone() else {
            self.simulation = None;
            return;
        };
        let products = self.compile_state.products.clone();
        if products.errors.is_empty() {
            let binary = match compiler::serialize(products.clone(), &module_name) {
                Ok(binary) => binary,
                Err(_) => {
                    self.simulation = None;
                    return;
                }
            };
            let vm = match vm::types::Module::new(binary) {
                Ok(vm) => vm,
                Err(_) => {
                    self.simulation = None;
                    return;
                }
            };
            let input_count = vm.inputs as usize;
            let output_count = vm.outputs.len();
            let (input_labels, output_labels) =
                module_labels(&products, &module_name, input_count, output_count);
            let wave_labels = input_labels
                .iter()
                .chain(output_labels.iter())
                .cloned()
                .collect::<Vec<_>>();
            let outputs = vm
                .get_output()
                .unwrap_or_else(|_| vec![false; output_count]);
            let circuit_gate_layout = products
                .expanded_modules
                .get(&module_name)
                .map(|compiled| {
                    optimize_circuit_graph_gate_layout(compiled, input_count, output_count)
                })
                .unwrap_or_else(|| CircuitGraphGateLayout {
                    gate_levels: Vec::new(),
                    gate_ranks: Vec::new(),
                    level_counts: vec![0],
                    max_level: 0,
                });
            self.simulation = Some(SimulationState {
                module_name: module_name.clone(),
                vm,
                inputs: vec![false; input_count],
                outputs,
                input_labels,
                output_labels,
                wave_labels: wave_labels.clone(),
                wave_data: vec![Vec::new(); wave_labels.len()],
                graphical: graphical_for_module(&products, &module_name),
                circuit_gate_layout,
            });
        } else {
            self.simulation = None;
        }
    }

    fn reset_vm(&mut self) {
        if let Some(simulation) = &mut self.simulation {
            simulation.vm.reset();
            simulation.inputs.fill(false);
            simulation.outputs = simulation
                .vm
                .get_output()
                .unwrap_or_else(|_| vec![false; simulation.output_labels.len()]);
            for channel in &mut simulation.wave_data {
                channel.clear();
            }
        }
    }

    fn tick_vm(&mut self) {
        let Some(simulation) = &mut self.simulation else {
            return;
        };
        for (index, value) in simulation.inputs.iter().copied().enumerate() {
            let _ = simulation.vm.set(index as u32, value);
        }
        let _ = simulation.vm.next(1);
        simulation.outputs = simulation
            .vm
            .get_output()
            .unwrap_or_else(|_| vec![false; simulation.output_labels.len()]);
        for (index, value) in simulation.inputs.iter().copied().enumerate() {
            simulation.wave_data[index].push(value);
        }
        for (index, value) in simulation.outputs.iter().copied().enumerate() {
            simulation.wave_data[index + simulation.inputs.len()].push(value);
        }
    }

    fn drive_auto_run(&mut self, ctx: &egui::Context) {
        if !self.auto_run {
            return;
        }
        let now = ctx.input(|input| input.time);
        let delay_seconds = (self.auto_run_delay_ms.max(1) as f64) / 1000.0;
        if now - self.last_auto_step_at >= delay_seconds {
            self.tick_vm();
            self.last_auto_step_at = now;
        }
        ctx.request_repaint_after(Duration::from_millis(self.auto_run_delay_ms.max(1)));
    }

    fn init_socket_bridge(&mut self, _ctx: &egui::Context) {
        #[cfg(target_arch = "wasm32")]
        if let Some(url) = socket_url_from_query() {
            self.socket_status = Some(format!("socket: connecting {url}"));
            connect_socket(url, _ctx.clone());
        }
    }

    fn poll_socket_bridge(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            SOCKET_INBOX.with(|inbox| {
                let mut inbox = inbox.borrow_mut();
                if let Some(status) = inbox.latest_status.clone() {
                    self.socket_status = Some(status);
                }
                if let Some(source) = inbox.latest_source.take() {
                    self.source = source;
                    self.recompile();
                }
            });
        }
    }

    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Compile").clicked() {
                    self.recompile();
                }
                if ui.button("Sample").clicked() {
                    self.source = SAMPLE_SOURCE.to_string();
                    self.recompile();
                }
                ui.checkbox(&mut self.auto_compile, "Auto compile");
                ui.separator();

                let mut module_changed = false;
                let selected_text = self.selected_module.as_deref().unwrap_or("-");
                egui::ComboBox::from_id_salt("module_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for module in &self.compile_state.products.module_type_list {
                            if ui
                                .selectable_value(
                                    &mut self.selected_module,
                                    Some(module.name.clone()),
                                    &module.name,
                                )
                                .changed()
                            {
                                module_changed = true;
                            }
                        }
                    });
                if module_changed {
                    self.rebuild_simulation();
                }

                ui.separator();
                if self.show_simulation_controls {
                    if ui.button("Hide controls").clicked() {
                        self.show_simulation_controls = false;
                    }
                    ui.checkbox(&mut self.auto_run, "Run");
                    ui.add(egui::Slider::new(&mut self.auto_run_delay_ms, 16..=1000).suffix(" ms"));
                    if ui.button("Step").clicked() {
                        self.tick_vm();
                    }
                    if ui.button("Reset").clicked() {
                        self.reset_vm();
                    }
                    ui.separator();
                    ui.add(egui::Slider::new(&mut self.logic_window, 8..=512).text("Logic window"));
                } else if ui.button("Show controls").clicked() {
                    self.show_simulation_controls = true;
                }

                let error_count = self.compile_state.products.errors.len()
                    + self.compile_state.tests.errors.len();
                let warn_count =
                    self.compile_state.products.warns.len() + self.compile_state.tests.warns.len();
                ui.separator();
                ui.label(format!("warnings: {warn_count}"));
                ui.label(format!("errors: {error_count}"));
                if let Some(status) = &self.socket_status {
                    ui.separator();
                    ui.label(status);
                }
            });
        });
    }

    fn show_editor(&mut self, ctx: &egui::Context) {
        const COLLAPSED_WIDTH: f32 = 28.0;

        if self.show_source_panel {
            let response = egui::SidePanel::left("editor")
                .default_width(self.source_panel_width)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Source");
                        if ui.small_button("Hide").clicked() {
                            self.show_source_panel = false;
                        }
                    });
                    ui.label("NCG editor");
                    let editor = egui::TextEdit::multiline(&mut self.source)
                        .font(TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .lock_focus(true);
                    let response = ui.add_sized(ui.available_size(), editor);
                    if response.changed() && self.auto_compile {
                        self.recompile();
                    }
                });
            self.source_panel_width = response.response.rect.width().max(220.0);
        } else {
            egui::SidePanel::left("editor")
                .min_width(COLLAPSED_WIDTH)
                .max_width(COLLAPSED_WIDTH)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        if ui.small_button(">").clicked() {
                            self.show_source_panel = true;
                        }
                        ui.small("Src");
                    });
                });
        }
    }

    fn show_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let full_rect = ui.available_rect_before_wrap();
            let total_width = full_rect.width();
            let total_height = full_rect.height();
            let separator_size = 8.0;
            let collapsed_side_width = 32.0;
            let collapsed_transpile_height = 28.0;
            let min_main_height = 120.0;
            let min_transpile_height = 120.0;
            let max_transpile_height =
                (total_height - min_main_height - separator_size).max(min_transpile_height);
            self.transpile_panel_height = self
                .transpile_panel_height
                .clamp(min_transpile_height, max_transpile_height);

            let main_height = if self.show_transpile_panel {
                (total_height - self.transpile_panel_height - separator_size).max(min_main_height)
            } else {
                (total_height - collapsed_transpile_height).max(0.0)
            };
            ui.allocate_ui_with_layout(
                vec2(total_width, main_height),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if self.show_diagnostics_panel {
                        let min_column_width =
                            ((total_width - separator_size) * 0.5).min(320.0).max(180.0);
                        let max_simulation_width =
                            (total_width - min_column_width - separator_size).max(min_column_width);
                        self.simulation_panel_width = self
                            .simulation_panel_width
                            .clamp(min_column_width, max_simulation_width);
                        let diagnostics_width =
                            (total_width - self.simulation_panel_width - separator_size)
                                .max(min_column_width);

                        ui.allocate_ui_with_layout(
                            vec2(self.simulation_panel_width, main_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.show_simulation_column(ui, main_height);
                            },
                        );

                        let (column_resize_rect, column_resize_response) = ui
                            .allocate_exact_size(vec2(separator_size, main_height), Sense::drag());
                        if column_resize_response.dragged() {
                            if let Some(pointer) = column_resize_response.interact_pointer_pos() {
                                self.simulation_panel_width = (pointer.x - full_rect.left())
                                    .clamp(min_column_width, max_simulation_width);
                                ctx.request_repaint();
                            }
                        }
                        let column_stroke = if column_resize_response.dragged()
                            || column_resize_response.hovered()
                        {
                            ui.style().visuals.widgets.active.fg_stroke
                        } else {
                            ui.style().visuals.widgets.noninteractive.bg_stroke
                        };
                        ui.painter().vline(
                            column_resize_rect.center().x,
                            column_resize_rect.y_range(),
                            column_stroke,
                        );

                        ui.allocate_ui_with_layout(
                            vec2(diagnostics_width, main_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.show_analysis_column(ui);
                            },
                        );
                    } else {
                        let simulation_width = (total_width - collapsed_side_width).max(120.0);
                        ui.allocate_ui_with_layout(
                            vec2(simulation_width, main_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.show_simulation_column(ui, main_height);
                            },
                        );
                        ui.allocate_ui_with_layout(
                            vec2(collapsed_side_width, main_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.vertical_centered(|ui| {
                                    if ui.small_button("<").clicked() {
                                        self.show_diagnostics_panel = true;
                                    }
                                    ui.small("Diag");
                                });
                            },
                        );
                    }
                },
            );

            if self.show_transpile_panel {
                let (resize_rect, resize_response) =
                    ui.allocate_exact_size(vec2(total_width, separator_size), Sense::drag());
                if resize_response.dragged() {
                    if let Some(pointer) = resize_response.interact_pointer_pos() {
                        self.transpile_panel_height = (full_rect.bottom() - pointer.y)
                            .clamp(min_transpile_height, max_transpile_height);
                        ctx.request_repaint();
                    }
                }
                let stroke = if resize_response.dragged() || resize_response.hovered() {
                    ui.style().visuals.widgets.active.fg_stroke
                } else {
                    ui.style().visuals.widgets.noninteractive.bg_stroke
                };
                ui.painter()
                    .hline(resize_rect.x_range(), resize_rect.center().y, stroke);

                ui.allocate_ui_with_layout(
                    vec2(total_width, self.transpile_panel_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("TypeScript transpile");
                            if ui.small_button("Hide").clicked() {
                                self.show_transpile_panel = false;
                            }
                        });
                        ui.separator();
                        let output =
                            egui::TextEdit::multiline(&mut self.compile_state.transpiled_ts)
                                .font(TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false);
                        ui.add_sized(ui.available_size(), output);
                    },
                );
            } else {
                ui.allocate_ui_with_layout(
                    vec2(total_width, collapsed_transpile_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.strong("TypeScript transpile");
                        ui.label("hidden");
                        if ui.small_button("Show").clicked() {
                            self.show_transpile_panel = true;
                        }
                    },
                );
            }
        });
    }

    fn show_analysis_column(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("analysis_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Diagnostics");
                    if ui.small_button("Hide").clicked() {
                        self.show_diagnostics_panel = false;
                    }
                });
                if self.compile_state.products.errors.is_empty()
                    && self.compile_state.products.warns.is_empty()
                    && self.compile_state.tests.errors.is_empty()
                    && self.compile_state.tests.warns.is_empty()
                {
                    ui.label("No warnings or errors.");
                }
                for error in &self.compile_state.products.errors {
                    diagnostic_label(ui, error, DiagnosticLevel::Error);
                }
                for warning in &self.compile_state.products.warns {
                    diagnostic_label(ui, warning, DiagnosticLevel::Warn);
                }
                for error in &self.compile_state.tests.errors {
                    diagnostic_label(ui, error, DiagnosticLevel::Error);
                }
                for warning in &self.compile_state.tests.warns {
                    diagnostic_label(ui, warning, DiagnosticLevel::Warn);
                }

                ui.separator();
                ui.heading("Module info");
                egui::Grid::new("module_info").striped(true).show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Type");
                    ui.strong("NOR gates");
                    ui.end_row();
                    for module in &self.compile_state.products.module_type_list {
                        let gate_count = self
                            .compile_state
                            .products
                            .expanded_modules
                            .get(&module.name)
                            .map(|expanded| expanded.gates_sequential.len())
                            .unwrap_or(0);
                        ui.label(&module.name);
                        ui.label(format!(
                            "{} -> {}",
                            module.mtype.input_count, module.mtype.output_count
                        ));
                        ui.label(gate_count.to_string());
                        ui.end_row();
                    }
                });

                ui.separator();
                ui.heading("Tests");
                if self.compile_state.tests.test_result.is_empty() {
                    ui.label("No test results.");
                } else {
                    for test_name in &self.compile_state.tests.test_list {
                        if let Some(patterns) = self.compile_state.tests.test_result.get(test_name)
                        {
                            let accept = patterns.iter().all(|pattern| pattern.accept);
                            let title = if accept {
                                RichText::new(format!("{test_name}  accept"))
                                    .color(Color32::LIGHT_GREEN)
                            } else {
                                RichText::new(format!("{test_name}  failed"))
                                    .color(Color32::LIGHT_RED)
                            };
                            egui::CollapsingHeader::new(title)
                                .default_open(!accept)
                                .show(ui, |ui| {
                                    egui::Grid::new(format!("test_grid_{test_name}"))
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.strong("accept");
                                            ui.strong("input");
                                            ui.strong("output");
                                            ui.strong("expect");
                                            ui.end_row();
                                            for pattern in patterns {
                                                let color = if pattern.accept {
                                                    Color32::LIGHT_GREEN
                                                } else {
                                                    Color32::LIGHT_RED
                                                };
                                                ui.colored_label(color, pattern.accept.to_string());
                                                ui.monospace(format_bits(&pattern.input));
                                                ui.monospace(format_bits(&pattern.output));
                                                ui.monospace(format_bits(&pattern.expect));
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    }
                }
            });
    }

    fn show_simulation_column(&mut self, ui: &mut egui::Ui, total_height: f32) {
        let show_graph_panel = self.show_circuit_graph && self.simulation.is_some();
        let simulation_title = self
            .simulation
            .as_ref()
            .map(|simulation| format!("Simulation: {}", simulation.module_name))
            .unwrap_or_else(|| "Simulation".to_string());
        let collapsed_height = 28.0;
        if !show_graph_panel {
            if self.show_simulation_details {
                egui::ScrollArea::vertical()
                    .id_salt("simulation_scroll")
                    .show(ui, |ui| {
                        self.show_simulation_content(ui);
                    });
            } else {
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), collapsed_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.strong(&simulation_title);
                        ui.label("hidden");
                        if ui.small_button("Show").clicked() {
                            self.show_simulation_details = true;
                        }
                    },
                );
            }
            return;
        }

        let full_rect = ui.available_rect_before_wrap();
        let total_width = full_rect.width();
        let separator_size = 8.0;
        if !self.show_simulation_details {
            let graph_height = (total_height - collapsed_height).max(120.0);
            ui.allocate_ui_with_layout(
                vec2(total_width, collapsed_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.strong(&simulation_title);
                    ui.label("hidden");
                    if ui.small_button("Show").clicked() {
                        self.show_simulation_details = true;
                    }
                },
            );
            ui.allocate_ui_with_layout(
                vec2(total_width, graph_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.show_circuit_graph_panel(ui);
                },
            );
            return;
        }
        let min_top_height = ((total_height - separator_size) * 0.5)
            .min(260.0)
            .max(160.0);
        let min_graph_height = ((total_height - separator_size) * 0.5)
            .min(320.0)
            .max(160.0);
        let max_graph_height =
            (total_height - min_top_height - separator_size).max(min_graph_height);
        self.circuit_graph_height = self
            .circuit_graph_height
            .clamp(min_graph_height, max_graph_height);
        let top_height =
            (total_height - self.circuit_graph_height - separator_size).max(min_top_height);

        ui.allocate_ui_with_layout(
            vec2(total_width, top_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("simulation_scroll")
                    .show(ui, |ui| {
                        self.show_simulation_content(ui);
                    });
            },
        );

        let (resize_rect, resize_response) =
            ui.allocate_exact_size(vec2(total_width, separator_size), Sense::drag());
        if resize_response.dragged() {
            if let Some(pointer) = resize_response.interact_pointer_pos() {
                self.circuit_graph_height =
                    (full_rect.bottom() - pointer.y).clamp(min_graph_height, max_graph_height);
                ui.ctx().request_repaint();
            }
        }
        let stroke = if resize_response.dragged() || resize_response.hovered() {
            ui.style().visuals.widgets.active.fg_stroke
        } else {
            ui.style().visuals.widgets.noninteractive.bg_stroke
        };
        ui.painter()
            .hline(resize_rect.x_range(), resize_rect.center().y, stroke);

        ui.allocate_ui_with_layout(
            vec2(total_width, self.circuit_graph_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                self.show_circuit_graph_panel(ui);
            },
        );
    }

    fn show_circuit_graph_panel(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_arch = "wasm32")]
        let mut export_clicked = false;
        ui.horizontal_wrapped(|ui| {
            ui.strong("Circuit graph");
            #[cfg(target_arch = "wasm32")]
            {
                ui.separator();
                ui.label("PNG");
                ui.add(
                    egui::DragValue::new(&mut self.circuit_graph_export_width)
                        .range(CIRCUIT_GRAPH_EXPORT_MIN_SIZE..=CIRCUIT_GRAPH_EXPORT_MAX_SIZE)
                        .speed(16.0)
                        .prefix("w "),
                );
                ui.add(
                    egui::DragValue::new(&mut self.circuit_graph_export_height)
                        .range(CIRCUIT_GRAPH_EXPORT_MIN_SIZE..=CIRCUIT_GRAPH_EXPORT_MAX_SIZE)
                        .speed(16.0)
                        .prefix("h "),
                );
                if ui.button("Download PNG").clicked() {
                    export_clicked = true;
                }
            }
        });

        #[cfg(target_arch = "wasm32")]
        if export_clicked {
            let width = self
                .circuit_graph_export_width
                .clamp(CIRCUIT_GRAPH_EXPORT_MIN_SIZE, CIRCUIT_GRAPH_EXPORT_MAX_SIZE);
            let height = self
                .circuit_graph_export_height
                .clamp(CIRCUIT_GRAPH_EXPORT_MIN_SIZE, CIRCUIT_GRAPH_EXPORT_MAX_SIZE);
            self.circuit_graph_export_status =
                Some(match self.export_circuit_graph_png(width, height) {
                    Ok(()) => format!("graph: downloaded {width}x{height} PNG"),
                    Err(error) => format!("graph: {error}"),
                });
        }

        if let Some(status) = &self.circuit_graph_export_status {
            ui.label(status);
        }
        ui.separator();
        if let Some(simulation) = self.simulation.as_ref() {
            if let Some(compiled) = self
                .compile_state
                .products
                .expanded_modules
                .get(&simulation.module_name)
            {
                draw_circuit_graph(ui, &self.compile_state.products, simulation, compiled);
            } else {
                ui.label("No expanded module available.");
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn export_circuit_graph_png(&self, width: u32, height: u32) -> Result<(), String> {
        let simulation = self
            .simulation
            .as_ref()
            .ok_or_else(|| "no simulation available".to_string())?;
        let compiled = self
            .compile_state
            .products
            .expanded_modules
            .get(&simulation.module_name)
            .ok_or_else(|| "no expanded module available".to_string())?;
        export_circuit_graph_png(
            &self.compile_state.products,
            simulation,
            compiled,
            width,
            height,
        )
    }

    fn show_simulation_content(&mut self, ui: &mut egui::Ui) {
        let Some(simulation) = &mut self.simulation else {
            ui.heading("Simulation");
            ui.label("Compile a valid module to inspect the VM.");
            return;
        };

        ui.horizontal(|ui| {
            ui.heading(format!("Simulation: {}", simulation.module_name));
            if ui.small_button("Hide").clicked() {
                self.show_simulation_details = false;
            }
        });
        egui::CollapsingHeader::new("Simulation state")
            .id_salt("simulation_state")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("tick: {}", simulation.vm.get_tick()));
            });

        egui::CollapsingHeader::new("Inputs")
            .id_salt("simulation_inputs")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, label) in simulation.input_labels.iter().enumerate() {
                        ui.checkbox(&mut simulation.inputs[index], label);
                    }
                });
            });

        egui::CollapsingHeader::new("Outputs")
            .id_salt("simulation_outputs")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (index, label) in simulation.output_labels.iter().enumerate() {
                        let mut value = simulation.outputs[index];
                        ui.add_enabled(false, egui::Checkbox::new(&mut value, label));
                    }
                });
            });

        egui::CollapsingHeader::new("Graphical I/O")
            .id_salt("simulation_graphical_io")
            .default_open(true)
            .show(ui, |ui| {
                if let Some(graphical) = simulation.graphical.clone() {
                    draw_graphical_io(ui, simulation, &graphical);
                } else {
                    ui.label("No graphical definition for this module.");
                }
            });

        egui::CollapsingHeader::new("Logic analyzer")
            .id_salt("simulation_logic_analyzer")
            .default_open(true)
            .show(ui, |ui| {
                draw_logic_analyzer(ui, simulation, self.logic_window);
            });

        egui::CollapsingHeader::new("Circuit graph")
            .id_salt("simulation_circuit_graph")
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.show_circuit_graph, "show");
                if self.show_circuit_graph {
                    ui.label("Drag the splitter below to resize the graph area.");
                } else {
                    ui.label("Circuit graph rendering is disabled.");
                }
            });
    }
}

impl eframe::App for CircuitGameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_socket_bridge();
        self.drive_auto_run(ctx);
        self.show_toolbar(ctx);
        self.show_editor(ctx);
        self.show_main(ctx);
    }
}

impl CompileState {
    fn empty() -> Self {
        Self {
            products: compiler::types::IntermediateProducts {
                source: String::new(),
                warns: Vec::new(),
                errors: Vec::new(),
                ast: compiler::types::File {
                    components: Vec::new(),
                },
                defined_non_func_module_list: Vec::new(),
                defined_func_module_list: Vec::new(),
                module_type_list: Vec::new(),
                module_dependency: Vec::new(),
                module_dependency_sorted: Vec::new(),
                expanded_modules: HashMap::new(),
            },
            tests: empty_test_products(),
            transpiled_ts: String::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn export_circuit_graph_png_from_source(
    input: &str,
    module_name: Option<&str>,
    output_path: &str,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let products = compiler::intermediate_products(input);
    if !products.errors.is_empty() {
        return Err(products.errors.join("\n"));
    }

    let module_name = resolve_circuit_graph_module_name(&products, module_name)?;
    let binary = compiler::serialize(products.clone(), &module_name)?;
    let vm = vm::types::Module::new(binary).map_err(|error| format!("failed to init VM: {error}"))?;
    let input_count = vm.inputs as usize;
    let output_count = vm.outputs.len();
    let (input_labels, output_labels) =
        module_labels(&products, &module_name, input_count, output_count);
    let wave_labels = input_labels
        .iter()
        .chain(output_labels.iter())
        .cloned()
        .collect::<Vec<_>>();
    let outputs = vm
        .get_output()
        .unwrap_or_else(|_| vec![false; output_count]);
    let circuit_gate_layout = products
        .expanded_modules
        .get(&module_name)
        .map(|compiled| optimize_circuit_graph_gate_layout(compiled, input_count, output_count))
        .unwrap_or_else(|| CircuitGraphGateLayout {
            gate_levels: Vec::new(),
            gate_ranks: Vec::new(),
            level_counts: vec![0],
            max_level: 0,
        });
    let simulation = SimulationState {
        module_name: module_name.clone(),
        vm,
        inputs: vec![false; input_count],
        outputs,
        input_labels,
        output_labels,
        wave_labels: wave_labels.clone(),
        wave_data: vec![Vec::new(); wave_labels.len()],
        graphical: graphical_for_module(&products, &module_name),
        circuit_gate_layout,
    };
    let compiled = products
        .expanded_modules
        .get(&module_name)
        .ok_or_else(|| format!("expanded module not found: {module_name}"))?;
    export_circuit_graph_png_native(
        &products,
        &simulation,
        compiled,
        width
            .clamp(CIRCUIT_GRAPH_EXPORT_MIN_SIZE, CIRCUIT_GRAPH_EXPORT_MAX_SIZE),
        height
            .clamp(CIRCUIT_GRAPH_EXPORT_MIN_SIZE, CIRCUIT_GRAPH_EXPORT_MAX_SIZE),
        output_path,
    )?;
    Ok(module_name)
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_circuit_graph_module_name(
    products: &IntermediateProducts,
    requested: Option<&str>,
) -> Result<String, String> {
    if let Some(requested) = requested {
        if products.expanded_modules.contains_key(requested) {
            return Ok(requested.to_string());
        }
        return Err(format!("module not found: {requested}"));
    }

    products
        .module_dependency_sorted
        .first()
        .cloned()
        .or_else(|| {
            products
                .module_type_list
                .first()
                .map(|module| module.name.clone())
        })
        .ok_or_else(|| "no modules available".to_string())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("window not available");
        let document = window.document().expect("document not available");
        let canvas = document
            .get_element_by_id(CANVAS_ID)
            .expect("canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas element type mismatch");
        let web_options = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(CircuitGameApp::new(cc)))),
            )
            .await
            .expect("failed to start circuitgame egui");
    });
}

#[cfg(target_arch = "wasm32")]
fn socket_url_from_query() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;
    let query = search.strip_prefix('?').unwrap_or(&search);
    for pair in query.split('&') {
        let mut split = pair.splitn(2, '=');
        if split.next()? == "socket" {
            let value = split.next()?.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn set_socket_status(status: String) {
    SOCKET_INBOX.with(|inbox| {
        inbox.borrow_mut().latest_status = Some(status);
    });
}

#[cfg(target_arch = "wasm32")]
fn set_socket_source(source: String) {
    SOCKET_INBOX.with(|inbox| {
        inbox.borrow_mut().latest_source = Some(source);
    });
}

#[cfg(target_arch = "wasm32")]
fn connect_socket(url: String, ctx: egui::Context) {
    let socket = match web_sys::WebSocket::new(&url) {
        Ok(socket) => socket,
        Err(error) => {
            set_socket_status(format!("socket: failed to open {url} ({error:?})"));
            ctx.request_repaint();
            return;
        }
    };

    set_socket_status(format!("socket: connecting {url}"));
    ctx.request_repaint();
    SOCKET_RECONNECT.with(|reconnect| {
        reconnect.borrow_mut().take();
    });

    let open_url = url.clone();
    let open_socket = socket.clone();
    let open_ctx = ctx.clone();
    let on_open = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let _ = open_socket.send_with_str("get file");
        set_socket_status(format!("socket: connected {open_url}"));
        open_ctx.request_repaint();
    }) as Box<dyn FnMut(_)>);
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    let message_url = url.clone();
    let message_socket = socket.clone();
    let message_ctx = ctx.clone();
    let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let Some(message) = event.data().as_string() else {
            return;
        };
        let Some(source) = message.strip_prefix("file:") else {
            return;
        };
        if source.is_empty() {
            let _ = message_socket.send_with_str("get file");
            return;
        }
        set_socket_source(format!("# received from {message_url}\n\n{source}"));
        set_socket_status(format!("socket: synced {message_url}"));
        message_ctx.request_repaint();
    }) as Box<dyn FnMut(_)>);
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    let error_url = url.clone();
    let error_ctx = ctx.clone();
    let on_error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        set_socket_status(format!("socket: error {error_url}"));
        error_ctx.request_repaint();
    }) as Box<dyn FnMut(_)>);
    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let close_url = url.clone();
    let close_ctx = ctx.clone();
    let on_close = Closure::wrap(Box::new(move |_event: web_sys::CloseEvent| {
        set_socket_status(format!("socket: reconnecting {close_url}"));
        close_ctx.request_repaint();
        schedule_socket_reconnect(close_url.clone(), close_ctx.clone());
    }) as Box<dyn FnMut(_)>);
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    SOCKET_RUNTIME.with(|runtime| {
        *runtime.borrow_mut() = Some(SocketRuntime {
            _socket: socket,
            _on_open: on_open,
            _on_message: on_message,
            _on_error: on_error,
            _on_close: on_close,
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn schedule_socket_reconnect(url: String, ctx: egui::Context) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let reconnect = Closure::wrap(Box::new(move || {
        connect_socket(url.clone(), ctx.clone());
    }) as Box<dyn FnMut()>);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        reconnect.as_ref().unchecked_ref(),
        3000,
    );
    SOCKET_RECONNECT.with(|slot| {
        *slot.borrow_mut() = Some(reconnect);
    });
}

fn empty_test_products() -> TestProducts {
    TestProducts {
        warns: Vec::new(),
        errors: Vec::new(),
        test_list: Vec::new(),
        test_result: HashMap::new(),
    }
}

fn transpile_all_modules(products: &IntermediateProducts) -> String {
    if !products.errors.is_empty() {
        return format!("// Error:\n{}", products.errors.join("\n"));
    }
    let module_names = products
        .defined_func_module_list
        .iter()
        .chain(products.defined_non_func_module_list.iter())
        .cloned()
        .collect::<Vec<_>>();
    let mut modules = Vec::new();
    for module_name in module_names {
        let binary = match compiler::serialize(products.clone(), &module_name) {
            Ok(binary) => binary,
            Err(err) => return format!("// Error: {err}"),
        };
        match vm::types::Module::new(binary) {
            Ok(module) => modules.push(module),
            Err(err) => return format!("// Error: {err}"),
        }
    }
    match transpiler::ts_transpiler::transpile(modules, false) {
        Ok(text) => text,
        Err(err) => format!("// Error: {err}"),
    }
}

fn module_labels(
    products: &IntermediateProducts,
    module_name: &str,
    input_count: usize,
    output_count: usize,
) -> (Vec<String>, Vec<String>) {
    if module_name == "nor" {
        return (
            vec!["x".to_string(), "y".to_string()],
            vec!["a".to_string()],
        );
    }
    for component in &products.ast.components {
        if let Component::Module(module) = component {
            if module.name == module_name {
                return (module.inputs.clone(), module.outputs.clone());
            }
        }
    }
    (
        (0..input_count).map(|index| format!("in{index}")).collect(),
        (0..output_count)
            .map(|index| format!("out{index}"))
            .collect(),
    )
}

fn graphical_for_module(products: &IntermediateProducts, module_name: &str) -> Option<Graphical> {
    products.ast.components.iter().find_map(|component| {
        if let Component::Graphical(graphical) = component {
            if graphical.name == module_name {
                return Some(graphical.clone());
            }
        }
        None
    })
}

fn graphical_dimensions(graphical: &Graphical) -> (u32, u32) {
    match graphical.size {
        ImgSize::Size { width, height } => (width.max(1), height.max(1)),
        ImgSize::Auto(_) => {
            let width = graphical
                .pixels
                .iter()
                .map(|pixel| pixel.coord.0)
                .max()
                .unwrap_or(0)
                + 1;
            let height = graphical
                .pixels
                .iter()
                .map(|pixel| pixel.coord.1)
                .max()
                .unwrap_or(0)
                + 1;
            (width.max(1), height.max(1))
        }
    }
}

fn draw_graphical_io(ui: &mut egui::Ui, simulation: &mut SimulationState, graphical: &Graphical) {
    let (width, height) = graphical_dimensions(graphical);
    let pixel_size = (ui.available_width() / width as f32).clamp(10.0, 28.0);
    let desired_size = vec2(width as f32 * pixel_size, height as f32 * pixel_size);
    let (response, painter) = ui.allocate_painter(desired_size, Sense::click());
    let rect = response.rect;
    painter.rect_filled(rect, 6.0, Color32::from_rgb(8, 8, 8));

    for pixel in &graphical.pixels {
        let x = rect.left() + pixel.coord.0 as f32 * pixel_size;
        let y = rect.top() + pixel.coord.1 as f32 * pixel_size;
        let pixel_rect = egui::Rect::from_min_size(pos2(x, y), vec2(pixel_size, pixel_size));
        let active = match pixel.io_index.io_type.as_str() {
            "input" => simulation
                .inputs
                .get(pixel.io_index.index as usize)
                .copied()
                .unwrap_or(false),
            "output" => simulation
                .outputs
                .get(pixel.io_index.index as usize)
                .copied()
                .unwrap_or(false),
            _ => false,
        };
        let color = if active {
            rgb_tuple(pixel.color.on)
        } else {
            rgb_tuple(pixel.color.off)
        };
        painter.rect_filled(pixel_rect, 0.0, color);
    }

    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let local_x = ((pointer.x - rect.left()) / pixel_size).floor().max(0.0) as u32;
            let local_y = ((pointer.y - rect.top()) / pixel_size).floor().max(0.0) as u32;
            for pixel in &graphical.pixels {
                if pixel.coord == (local_x, local_y) && pixel.io_index.io_type == "input" {
                    if let Some(input) = simulation.inputs.get_mut(pixel.io_index.index as usize) {
                        *input = !*input;
                    }
                }
            }
        }
    }
}

fn draw_logic_analyzer(ui: &mut egui::Ui, simulation: &SimulationState, logic_window: usize) {
    let sample_count = simulation
        .wave_data
        .iter()
        .map(|channel| channel.len())
        .max()
        .unwrap_or(0);
    if sample_count == 0 {
        ui.label("No waveform yet. Step the VM to capture samples.");
        return;
    }

    let label_width = 88.0;
    let channel_height = 28.0;
    let desired_size = vec2(
        ui.available_width().max(420.0),
        (simulation.wave_labels.len() as f32 * channel_height + 24.0).max(140.0),
    );
    let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
    let rect = response.rect;
    painter.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0, Color32::from_gray(70)),
        egui::StrokeKind::Inside,
    );

    let start = sample_count.saturating_sub(logic_window.max(2));
    let visible_samples = sample_count - start;
    let step_width = ((rect.width() - label_width - 12.0) / visible_samples.max(1) as f32).max(4.0);
    let plot_left = rect.left() + label_width;
    let plot_right = rect.right() - 8.0;
    let grid_stride = ((visible_samples as f32) / 10.0).ceil() as usize;
    let grid_stride = grid_stride.max(1);

    for sample_index in 0..visible_samples {
        if sample_index % grid_stride != 0 {
            continue;
        }
        let x = plot_left + sample_index as f32 * step_width;
        painter.line_segment(
            [pos2(x, rect.top() + 6.0), pos2(x, rect.bottom() - 6.0)],
            Stroke::new(1.0, Color32::from_gray(55)),
        );
        painter.text(
            pos2(x, rect.top() + 2.0),
            Align2::CENTER_TOP,
            format!("{}", start + sample_index),
            FontId::monospace(11.0),
            Color32::GRAY,
        );
    }

    for (channel_index, label) in simulation.wave_labels.iter().enumerate() {
        let row_top = rect.top() + 20.0 + channel_height * channel_index as f32;
        let row_mid = row_top + channel_height * 0.5;
        let row_high = row_top + channel_height * 0.28;
        let row_low = row_top + channel_height * 0.72;
        let color = if channel_index < simulation.inputs.len() {
            Color32::from_rgb(70, 170, 255)
        } else {
            Color32::from_rgb(255, 170, 80)
        };

        painter.text(
            pos2(rect.left() + 8.0, row_mid),
            Align2::LEFT_CENTER,
            label,
            FontId::monospace(12.0),
            Color32::WHITE,
        );
        painter.line_segment(
            [pos2(plot_left, row_mid), pos2(plot_right, row_mid)],
            Stroke::new(1.0, Color32::from_gray(45)),
        );

        let channel = &simulation.wave_data[channel_index][start..];
        if channel.is_empty() {
            continue;
        }
        for index in 0..channel.len() {
            let x0 = plot_left + index as f32 * step_width;
            let x1 = (x0 + step_width).min(plot_right);
            let y = if channel[index] { row_high } else { row_low };
            painter.line_segment([pos2(x0, y), pos2(x1, y)], Stroke::new(2.0, color));
            if index + 1 < channel.len() && channel[index] != channel[index + 1] {
                let next_y = if channel[index + 1] {
                    row_high
                } else {
                    row_low
                };
                painter.line_segment([pos2(x1, y), pos2(x1, next_y)], Stroke::new(2.0, color));
            }
        }
    }
}

fn circuit_graph_content_rect(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        pos2(
            rect.left() + CIRCUIT_GRAPH_HORIZONTAL_PADDING,
            rect.top() + CIRCUIT_GRAPH_VERTICAL_PADDING_TOP,
        ),
        pos2(
            rect.right() - CIRCUIT_GRAPH_HORIZONTAL_PADDING,
            rect.bottom() - CIRCUIT_GRAPH_VERTICAL_PADDING_BOTTOM,
        ),
    )
}

fn build_circuit_graph_render_scene(
    products: &IntermediateProducts,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
    rect: egui::Rect,
) -> CircuitGraphRenderScene {
    let gate_count = compiled.gates_sequential.len();
    let content_rect = circuit_graph_content_rect(rect);
    let input_positions = lane_positions(
        simulation.inputs.len(),
        content_rect.left() + 56.0,
        &content_rect,
    );
    let gate_positions = gate_positions(&content_rect, &simulation.circuit_gate_layout);
    let output_positions = lane_positions(
        simulation.outputs.len(),
        content_rect.right() - 56.0,
        &content_rect,
    );
    let cond = simulation.vm.get_gates();
    let gate_states = cond[..gate_count.min(cond.len())].to_vec();
    let gate_rect_template = egui::Rect::from_center_size(pos2(0.0, 0.0), vec2(44.0, 24.0));
    let group_layout =
        circuit_graph_group_for_module(products, &simulation.module_name).and_then(|group| {
            circuit_graph_group_layout(&group, &gate_positions, gate_rect_template, 0)
        });
    let label_layouts = group_layout
        .as_ref()
        .map(|layout| {
            optimize_circuit_graph_labels(layout, &gate_positions, rect, gate_rect_template)
        })
        .unwrap_or_default();

    CircuitGraphRenderScene {
        rect,
        input_positions,
        gate_positions,
        output_positions,
        gate_states,
        group_layout,
        label_layouts,
    }
}

fn draw_circuit_graph_scene(
    painter: &egui::Painter,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
    scene: &CircuitGraphRenderScene,
) {
    let gate_count = compiled.gates_sequential.len();
    if let Some(layout) = &scene.group_layout {
        draw_circuit_group(painter, layout, &scene.label_layouts);
    }

    for (gate_index, gate) in compiled.gates_sequential.iter().enumerate() {
        let gate_pos = scene
            .gate_positions
            .get(gate_index)
            .copied()
            .unwrap_or(scene.rect.center());
        for input in [&gate.0, &gate.1] {
            let (source_pos, active) = gate_source_position(
                input,
                &scene.input_positions,
                &scene.gate_positions,
                scene.rect.left() + CIRCUIT_GRAPH_HORIZONTAL_PADDING + 56.0,
                scene.rect.right() - CIRCUIT_GRAPH_HORIZONTAL_PADDING - 56.0,
                &simulation.inputs,
                &scene.gate_states,
            );
            draw_signal_edge(painter, source_pos, gate_pos, active);
        }
    }

    for (output_index, source) in compiled.outputs.iter().enumerate() {
        if let Some(target) = scene.output_positions.get(output_index).copied() {
            let (source_pos, active) = output_source_position(
                *source,
                gate_count,
                &scene.input_positions,
                &scene.gate_positions,
                scene.rect.left() + CIRCUIT_GRAPH_HORIZONTAL_PADDING + 56.0,
                scene.rect.right() - CIRCUIT_GRAPH_HORIZONTAL_PADDING - 56.0,
                &simulation.inputs,
                &scene.gate_states,
            );
            draw_signal_edge(painter, source_pos, target, active);
        }
    }

    for (index, position) in scene.input_positions.iter().enumerate() {
        let active = simulation.inputs.get(index).copied().unwrap_or(false);
        painter.circle_filled(*position, 10.0, node_color(active, NodeKind::Input));
        painter.text(
            pos2(position.x - 16.0, position.y),
            Align2::RIGHT_CENTER,
            simulation
                .input_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("in{index}")),
            FontId::monospace(11.0),
            Color32::WHITE,
        );
    }

    for (index, position) in scene.gate_positions.iter().enumerate() {
        let active = scene.gate_states.get(index).copied().unwrap_or(false);
        let gate_rect = egui::Rect::from_center_size(*position, vec2(36.0, 20.0));
        painter.rect_filled(gate_rect, 4.0, node_color(active, NodeKind::Gate));
        painter.rect_stroke(
            gate_rect,
            4.0,
            Stroke::new(1.0, Color32::from_gray(90)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            gate_rect.center(),
            Align2::CENTER_CENTER,
            "nor",
            FontId::monospace(10.0),
            Color32::WHITE,
        );
        painter.text(
            pos2(position.x, position.y - 16.0),
            Align2::CENTER_BOTTOM,
            format!("g{index}"),
            FontId::monospace(10.0),
            Color32::GRAY,
        );
    }

    for (index, position) in scene.output_positions.iter().enumerate() {
        let active = simulation.outputs.get(index).copied().unwrap_or(false);
        painter.rect_filled(
            egui::Rect::from_center_size(*position, vec2(20.0, 20.0)),
            3.0,
            node_color(active, NodeKind::Output),
        );
        painter.text(
            pos2(position.x + 16.0, position.y),
            Align2::LEFT_CENTER,
            simulation
                .output_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("out{index}")),
            FontId::monospace(11.0),
            Color32::WHITE,
        );
    }
}

fn draw_circuit_graph(
    ui: &mut egui::Ui,
    products: &IntermediateProducts,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
) {
    let row_count = compiled
        .gates_sequential
        .len()
        .max(simulation.inputs.len())
        .max(simulation.outputs.len())
        .max(1);
    let viewport_size = ui.available_size_before_wrap();
    let desired_size = vec2(
        viewport_size.x.max(620.0),
        ((row_count as f32 * 34.0)
            + CIRCUIT_GRAPH_VERTICAL_PADDING_TOP
            + CIRCUIT_GRAPH_VERTICAL_PADDING_BOTTOM)
            .max(viewport_size.y.max(180.0) - 8.0)
            .max(280.0),
    );
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .id_salt("circuit_graph_scroll")
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
            let scene =
                build_circuit_graph_render_scene(products, simulation, compiled, response.rect);
            painter.rect_stroke(
                scene.rect,
                6.0,
                Stroke::new(1.0, Color32::from_gray(70)),
                egui::StrokeKind::Inside,
            );
            draw_circuit_graph_scene(&painter, simulation, compiled, &scene);
        });
}

fn circuit_graph_group_for_module(
    products: &IntermediateProducts,
    module_name: &str,
) -> Option<CircuitGraphGroup> {
    let mut next_group_id = 0;
    let (group, _) = build_circuit_graph_group(products, module_name, 0, &mut next_group_id);
    group
}

fn build_circuit_graph_group(
    products: &IntermediateProducts,
    module_name: &str,
    mut gate_cursor: usize,
    next_group_id: &mut usize,
) -> (Option<CircuitGraphGroup>, usize) {
    if module_name == "nor" {
        return (None, gate_cursor.saturating_add(1));
    }
    let Some(module) = module_definition(products, module_name) else {
        return (None, gate_cursor);
    };

    let start = gate_cursor;
    let mut children = Vec::new();
    for gate in &module.gates {
        if gate.module_name == "nor" {
            gate_cursor += 1;
            continue;
        }
        let (child, next_gate_cursor) =
            build_circuit_graph_group(products, &gate.module_name, gate_cursor, next_group_id);
        gate_cursor = next_gate_cursor;
        if let Some(child) = child {
            children.push(child);
        }
    }

    if gate_cursor == start {
        return (None, gate_cursor);
    }

    (
        Some(CircuitGraphGroup {
            id: {
                let id = *next_group_id;
                *next_group_id += 1;
                id
            },
            label: module_name.to_string(),
            gate_range: start..gate_cursor,
            children,
        }),
        gate_cursor,
    )
}

fn module_definition<'a>(
    products: &'a IntermediateProducts,
    module_name: &str,
) -> Option<&'a compiler::types::Module> {
    products
        .ast
        .components
        .iter()
        .find_map(|component| match component {
            Component::Module(module) if module.name == module_name => Some(module),
            _ => None,
        })
}

fn gate_layout_levels(compiled: &compiler::types::CompiledModule) -> Vec<usize> {
    let mut levels = vec![1; compiled.gates_sequential.len()];
    for (gate_index, gate) in compiled.gates_sequential.iter().enumerate() {
        let mut level = 1;
        for input in [&gate.0, &gate.1] {
            let input_level = match input {
                CompiledGateInput::Input(_) => 0,
                CompiledGateInput::NorGate(index) => {
                    if (*index as usize) < gate_index {
                        levels[*index as usize] + 1
                    } else {
                        1
                    }
                }
            };
            level = level.max(input_level);
        }
        levels[gate_index] = level;
    }
    levels
}

fn default_circuit_graph_gate_layout(
    compiled: &compiler::types::CompiledModule,
) -> CircuitGraphGateLayout {
    let gate_levels = gate_layout_levels(compiled);
    let max_level = gate_levels.iter().copied().max().unwrap_or(0);
    let mut level_orders = vec![Vec::new(); max_level.saturating_add(1)];
    for (gate_index, level) in gate_levels.iter().copied().enumerate() {
        if level >= level_orders.len() {
            level_orders.resize(level + 1, Vec::new());
        }
        level_orders[level].push(gate_index);
    }
    circuit_graph_gate_layout_from_orders(&gate_levels, &level_orders, max_level)
}

fn optimize_circuit_graph_gate_layout(
    compiled: &compiler::types::CompiledModule,
    input_count: usize,
    output_count: usize,
) -> CircuitGraphGateLayout {
    if compiled.gates_sequential.is_empty() {
        return CircuitGraphGateLayout {
            gate_levels: Vec::new(),
            gate_ranks: Vec::new(),
            level_counts: vec![0],
            max_level: 0,
        };
    }

    let gate_levels = gate_layout_levels(compiled);
    let max_level = gate_levels.iter().copied().max().unwrap_or(0);
    let mut level_orders = vec![Vec::new(); max_level.saturating_add(1)];
    for (gate_index, level) in gate_levels.iter().copied().enumerate() {
        level_orders[level].push(gate_index);
    }

    for _ in 0..4 {
        let forward_layout =
            circuit_graph_gate_layout_from_orders(&gate_levels, &level_orders, max_level);
        for level in 1..=max_level {
            reorder_gate_level_by_barycenter(
                compiled,
                &forward_layout,
                input_count,
                output_count,
                &mut level_orders[level],
                true,
            );
        }

        let backward_layout =
            circuit_graph_gate_layout_from_orders(&gate_levels, &level_orders, max_level);
        for level in (1..=max_level).rev() {
            reorder_gate_level_by_barycenter(
                compiled,
                &backward_layout,
                input_count,
                output_count,
                &mut level_orders[level],
                false,
            );
        }
    }

    let mut best_layout =
        circuit_graph_gate_layout_from_orders(&gate_levels, &level_orders, max_level);
    let mut best_eval =
        evaluate_circuit_graph_wires(compiled, input_count, output_count, &best_layout);

    for _ in 0..3 {
        let mut improved = false;
        for level in 1..=max_level {
            if level_orders[level].len() < 2 {
                continue;
            }
            loop {
                let mut local_improved = false;
                for slot in 0..level_orders[level].len().saturating_sub(1) {
                    level_orders[level].swap(slot, slot + 1);
                    let candidate_layout = circuit_graph_gate_layout_from_orders(
                        &gate_levels,
                        &level_orders,
                        max_level,
                    );
                    let candidate_eval = evaluate_circuit_graph_wires(
                        compiled,
                        input_count,
                        output_count,
                        &candidate_layout,
                    );
                    if candidate_eval.total + 0.001 < best_eval.total {
                        best_layout = candidate_layout;
                        best_eval = candidate_eval;
                        improved = true;
                        local_improved = true;
                    } else {
                        level_orders[level].swap(slot, slot + 1);
                    }
                }
                if !local_improved {
                    break;
                }
            }
        }
        if !improved {
            break;
        }
    }

    best_layout
}

fn circuit_graph_gate_layout_from_orders(
    gate_levels: &[usize],
    level_orders: &[Vec<usize>],
    max_level: usize,
) -> CircuitGraphGateLayout {
    let mut gate_ranks = vec![0; gate_levels.len()];
    let mut level_counts = vec![0; max_level.saturating_add(1)];
    for (level, order) in level_orders.iter().enumerate() {
        if level < level_counts.len() {
            level_counts[level] = order.len();
        }
        for (rank, gate_index) in order.iter().copied().enumerate() {
            if gate_index < gate_ranks.len() {
                gate_ranks[gate_index] = rank;
            }
        }
    }
    CircuitGraphGateLayout {
        gate_levels: gate_levels.to_vec(),
        gate_ranks,
        level_counts,
        max_level,
    }
}

fn reorder_gate_level_by_barycenter(
    compiled: &compiler::types::CompiledModule,
    layout: &CircuitGraphGateLayout,
    input_count: usize,
    output_count: usize,
    gate_order: &mut [usize],
    use_inputs: bool,
) {
    let (input_positions, gate_positions, output_positions) =
        circuit_graph_normalized_positions(input_count, output_count, layout);
    let edges = circuit_graph_edge_refs(compiled);
    let mut keyed = gate_order
        .iter()
        .copied()
        .map(|gate_index| {
            let barycenter = if use_inputs {
                circuit_graph_inbound_barycenter(
                    gate_index,
                    &edges,
                    &input_positions,
                    &gate_positions,
                    &output_positions,
                )
            } else {
                circuit_graph_outbound_barycenter(
                    gate_index,
                    &edges,
                    &input_positions,
                    &gate_positions,
                    &output_positions,
                )
            }
            .unwrap_or_else(|| {
                gate_positions
                    .get(gate_index)
                    .map(|position| position.y)
                    .unwrap_or(0.5)
            });
            (gate_index, barycenter)
        })
        .collect::<Vec<_>>();
    keyed.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    for (index, (gate_index, _)) in keyed.into_iter().enumerate() {
        gate_order[index] = gate_index;
    }
}

fn circuit_graph_inbound_barycenter(
    gate_index: usize,
    edges: &[CircuitGraphEdgeRef],
    input_positions: &[egui::Pos2],
    gate_positions: &[egui::Pos2],
    output_positions: &[egui::Pos2],
) -> Option<f32> {
    let mut values = Vec::new();
    for edge in edges {
        if edge.target == CircuitGraphNodeRef::Gate(gate_index) {
            values.push(
                circuit_graph_node_position(
                    edge.source,
                    input_positions,
                    gate_positions,
                    output_positions,
                )
                .y,
            );
        }
    }
    average(values)
}

fn circuit_graph_outbound_barycenter(
    gate_index: usize,
    edges: &[CircuitGraphEdgeRef],
    input_positions: &[egui::Pos2],
    gate_positions: &[egui::Pos2],
    output_positions: &[egui::Pos2],
) -> Option<f32> {
    let mut values = Vec::new();
    for edge in edges {
        if edge.source == CircuitGraphNodeRef::Gate(gate_index) {
            values.push(
                circuit_graph_node_position(
                    edge.target,
                    input_positions,
                    gate_positions,
                    output_positions,
                )
                .y,
            );
        }
    }
    average(values)
}

fn average(values: Vec<f32>) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>() / values.len() as f32)
    }
}

fn gate_positions(rect: &egui::Rect, layout: &CircuitGraphGateLayout) -> Vec<egui::Pos2> {
    if layout.gate_levels.is_empty() {
        return Vec::new();
    }
    let left = rect.left() + 140.0;
    let right = (rect.right() - 140.0).max(left + 40.0);
    layout
        .gate_levels
        .iter()
        .copied()
        .enumerate()
        .map(|(gate_index, level)| {
            let t = if layout.max_level <= 1 {
                0.5
            } else {
                (level.saturating_sub(1)) as f32 / (layout.max_level.saturating_sub(1)) as f32
            };
            let x = egui::lerp(left..=right, t);
            let rank = layout.gate_ranks.get(gate_index).copied().unwrap_or(0);
            let count = layout.level_counts.get(level).copied().unwrap_or(1).max(1);
            pos2(
                x,
                vertical_coordinate(rank, count, rect.top() + 24.0, rect.bottom() - 24.0),
            )
        })
        .collect()
}

fn lane_positions(count: usize, x: f32, rect: &egui::Rect) -> Vec<egui::Pos2> {
    vertical_positions(count, rect)
        .into_iter()
        .map(|mut position| {
            position.x = x;
            position
        })
        .collect()
}

fn circuit_graph_edge_refs(compiled: &compiler::types::CompiledModule) -> Vec<CircuitGraphEdgeRef> {
    let gate_count = compiled.gates_sequential.len();
    let mut edges = Vec::new();
    for (gate_index, gate) in compiled.gates_sequential.iter().enumerate() {
        for input in [&gate.0, &gate.1] {
            edges.push(CircuitGraphEdgeRef {
                source: match input {
                    CompiledGateInput::Input(index) => CircuitGraphNodeRef::Input(*index as usize),
                    CompiledGateInput::NorGate(index) => CircuitGraphNodeRef::Gate(*index as usize),
                },
                target: CircuitGraphNodeRef::Gate(gate_index),
            });
        }
    }
    for (output_index, source) in compiled.outputs.iter().copied().enumerate() {
        edges.push(CircuitGraphEdgeRef {
            source: if (source as usize) < gate_count {
                CircuitGraphNodeRef::Gate(source as usize)
            } else {
                CircuitGraphNodeRef::Input(source as usize - gate_count)
            },
            target: CircuitGraphNodeRef::Output(output_index),
        });
    }
    edges
}

fn evaluate_circuit_graph_wires(
    compiled: &compiler::types::CompiledModule,
    input_count: usize,
    output_count: usize,
    layout: &CircuitGraphGateLayout,
) -> CircuitGraphWireEvaluation {
    let edges = circuit_graph_edge_refs(compiled);
    let (input_positions, gate_positions, output_positions) =
        circuit_graph_normalized_positions(input_count, output_count, layout);
    let curves = edges
        .iter()
        .map(|edge| {
            circuit_graph_curve_points(
                circuit_graph_node_position(
                    edge.source,
                    &input_positions,
                    &gate_positions,
                    &output_positions,
                ),
                circuit_graph_node_position(
                    edge.target,
                    &input_positions,
                    &gate_positions,
                    &output_positions,
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut evaluation = CircuitGraphWireEvaluation::default();
    evaluation.crossings = circuit_graph_curve_crossings(&curves);
    for edge in &edges {
        let source = circuit_graph_node_position(
            edge.source,
            &input_positions,
            &gate_positions,
            &output_positions,
        );
        let target = circuit_graph_node_position(
            edge.target,
            &input_positions,
            &gate_positions,
            &output_positions,
        );
        let dx = target.x - source.x;
        if dx.abs() < 0.05 {
            evaluation.same_level_penalty += 1.0;
        } else if dx < 0.0 {
            evaluation.backward_penalty += 1.0;
        }
    }
    evaluation.total = evaluation.crossings * 200.0
        + evaluation.same_level_penalty * 8.0
        + evaluation.backward_penalty * 40.0;
    evaluation
}

fn circuit_graph_normalized_positions(
    input_count: usize,
    output_count: usize,
    layout: &CircuitGraphGateLayout,
) -> (Vec<egui::Pos2>, Vec<egui::Pos2>, Vec<egui::Pos2>) {
    let input_positions = normalized_lane_positions(input_count, 0.0);
    let output_positions = normalized_lane_positions(output_count, layout.max_level as f32 + 1.0);
    let gate_positions = layout
        .gate_levels
        .iter()
        .copied()
        .enumerate()
        .map(|(gate_index, level)| {
            let rank = layout.gate_ranks.get(gate_index).copied().unwrap_or(0);
            let count = layout.level_counts.get(level).copied().unwrap_or(1).max(1);
            pos2(level as f32, normalized_vertical_coordinate(rank, count))
        })
        .collect::<Vec<_>>();
    (input_positions, gate_positions, output_positions)
}

fn normalized_lane_positions(count: usize, x: f32) -> Vec<egui::Pos2> {
    (0..count)
        .map(|index| pos2(x, normalized_vertical_coordinate(index, count.max(1))))
        .collect()
}

fn circuit_graph_node_position(
    node: CircuitGraphNodeRef,
    input_positions: &[egui::Pos2],
    gate_positions: &[egui::Pos2],
    output_positions: &[egui::Pos2],
) -> egui::Pos2 {
    match node {
        CircuitGraphNodeRef::Input(index) => input_positions
            .get(index)
            .copied()
            .unwrap_or(pos2(0.0, 0.5)),
        CircuitGraphNodeRef::Gate(index) => {
            gate_positions.get(index).copied().unwrap_or(pos2(0.0, 0.5))
        }
        CircuitGraphNodeRef::Output(index) => output_positions
            .get(index)
            .copied()
            .unwrap_or(pos2(0.0, 0.5)),
    }
}

fn circuit_graph_curve_points(source: egui::Pos2, target: egui::Pos2) -> [egui::Pos2; 4] {
    let dx = target.x - source.x;
    if dx >= 0.0 {
        let handle = dx * 0.45;
        [
            source,
            pos2(source.x + handle, source.y),
            pos2(target.x - handle, target.y),
            target,
        ]
    } else {
        let loop_height = 0.18 + dx.abs() * 0.2;
        [
            source,
            pos2(source.x + 0.35, source.y - loop_height),
            pos2(target.x - 0.35, target.y - loop_height),
            target,
        ]
    }
}

fn circuit_graph_curve_crossings(curves: &[[egui::Pos2; 4]]) -> f32 {
    let mut crossings = 0.0;
    for left_index in 0..curves.len() {
        for right_index in left_index + 1..curves.len() {
            if segments_cross(
                curves[left_index][0],
                curves[left_index][3],
                curves[right_index][0],
                curves[right_index][3],
            ) {
                crossings += 1.0;
            }
        }
    }
    crossings
}

fn segments_cross(a0: egui::Pos2, a1: egui::Pos2, b0: egui::Pos2, b1: egui::Pos2) -> bool {
    if points_close(a0, b0) || points_close(a0, b1) || points_close(a1, b0) || points_close(a1, b1)
    {
        return false;
    }
    let o1 = segment_orientation(a0, a1, b0);
    let o2 = segment_orientation(a0, a1, b1);
    let o3 = segment_orientation(b0, b1, a0);
    let o4 = segment_orientation(b0, b1, a1);
    (o1 > 0.0 && o2 < 0.0 || o1 < 0.0 && o2 > 0.0) && (o3 > 0.0 && o4 < 0.0 || o3 < 0.0 && o4 > 0.0)
}

fn segment_orientation(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn points_close(left: egui::Pos2, right: egui::Pos2) -> bool {
    (left.x - right.x).abs() < 0.0001 && (left.y - right.y).abs() < 0.0001
}

fn normalized_vertical_coordinate(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.5
    } else {
        index as f32 / (count - 1) as f32
    }
}

fn vertical_coordinate(index: usize, count: usize, top: f32, bottom: f32) -> f32 {
    egui::lerp(top..=bottom, normalized_vertical_coordinate(index, count))
}

fn circuit_graph_group_layout(
    group: &CircuitGraphGroup,
    gate_positions: &[egui::Pos2],
    gate_rect_template: egui::Rect,
    depth: usize,
) -> Option<CircuitGraphGroupLayout> {
    let rect = circuit_group_rect_for_range(
        group.gate_range.clone(),
        gate_positions,
        gate_rect_template,
        depth,
    )?;
    let children = group
        .children
        .iter()
        .filter_map(|child| {
            circuit_graph_group_layout(child, gate_positions, gate_rect_template, depth + 1)
        })
        .collect::<Vec<_>>();
    Some(CircuitGraphGroupLayout {
        id: group.id,
        label: group.label.clone(),
        depth,
        rect,
        children,
    })
}

fn circuit_group_rect_for_range(
    gate_range: Range<usize>,
    gate_positions: &[egui::Pos2],
    gate_rect_template: egui::Rect,
    depth: usize,
) -> Option<egui::Rect> {
    let positions = gate_positions
        .get(gate_range)
        .filter(|positions| !positions.is_empty())?;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for position in positions {
        min_x = min_x.min(position.x);
        min_y = min_y.min(position.y);
        max_x = max_x.max(position.x);
        max_y = max_y.max(position.y);
    }

    let padding_x = 28.0 + depth as f32 * 4.0;
    let padding_top = 24.0;
    let padding_bottom = 18.0 + depth as f32 * 2.0;
    Some(egui::Rect::from_min_max(
        pos2(
            min_x - gate_rect_template.width() * 0.5 - padding_x,
            min_y - gate_rect_template.height() * 0.5 - padding_top,
        ),
        pos2(
            max_x + gate_rect_template.width() * 0.5 + padding_x,
            max_y + gate_rect_template.height() * 0.5 + padding_x.min(padding_bottom),
        ),
    ))
}

fn collect_circuit_graph_group_layouts<'a>(
    group: &'a CircuitGraphGroupLayout,
    groups: &mut Vec<&'a CircuitGraphGroupLayout>,
) {
    groups.push(group);
    for child in &group.children {
        collect_circuit_graph_group_layouts(child, groups);
    }
}

fn collect_visible_circuit_graph_group_layouts<'a>(
    root: &'a CircuitGraphGroupLayout,
    groups: &mut Vec<&'a CircuitGraphGroupLayout>,
) {
    if root.depth == 0 {
        for child in &root.children {
            collect_circuit_graph_group_layouts(child, groups);
        }
    } else {
        collect_circuit_graph_group_layouts(root, groups);
    }
}

fn optimize_circuit_graph_labels(
    root: &CircuitGraphGroupLayout,
    gate_positions: &[egui::Pos2],
    graph_rect: egui::Rect,
    gate_rect_template: egui::Rect,
) -> HashMap<usize, CircuitGraphLabelPlacement> {
    let mut groups = Vec::new();
    collect_visible_circuit_graph_group_layouts(root, &mut groups);
    groups.sort_by(|left, right| {
        right.depth.cmp(&left.depth).then_with(|| {
            let left_area = (left.rect.width() * left.rect.height() * 100.0) as i64;
            let right_area = (right.rect.width() * right.rect.height() * 100.0) as i64;
            left_area.cmp(&right_area)
        })
    });

    let gate_obstacles = circuit_graph_gate_obstacles(gate_positions, gate_rect_template);
    let mut placements = HashMap::new();

    for group in &groups {
        let best = choose_best_circuit_graph_label(group, &placements, &gate_obstacles, graph_rect);
        placements.insert(group.id, best);
    }

    for _ in 0..6 {
        let mut changed = false;
        for group in &groups {
            let best =
                choose_best_circuit_graph_label(group, &placements, &gate_obstacles, graph_rect);
            let needs_update = placements.get(&group.id).map_or(true, |current| {
                current.rect != best.rect
                    || current.label != best.label
                    || current.anchor != best.anchor
            });
            if needs_update {
                placements.insert(group.id, best);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    placements
}

fn choose_best_circuit_graph_label(
    group: &CircuitGraphGroupLayout,
    placements: &HashMap<usize, CircuitGraphLabelPlacement>,
    gate_obstacles: &[egui::Rect],
    graph_rect: egui::Rect,
) -> CircuitGraphLabelPlacement {
    let mut best: Option<(CircuitGraphLabelPlacement, CircuitGraphLayoutEvaluation)> = None;
    for (anchor, base_cost, rect, label) in circuit_graph_label_candidates(group, graph_rect) {
        let score = evaluate_circuit_graph_label(
            group.id,
            &rect,
            base_cost,
            placements,
            gate_obstacles,
            graph_rect,
        );
        let candidate = CircuitGraphLabelPlacement {
            label,
            rect,
            anchor,
        };
        let should_replace = match best.as_ref() {
            Some((_, best_score)) => score.total < best_score.total,
            None => true,
        };
        if should_replace {
            best = Some((candidate, score));
        }
    }
    best.expect("at least one label candidate").0
}

fn circuit_graph_label_candidates(
    group: &CircuitGraphGroupLayout,
    graph_rect: egui::Rect,
) -> Vec<(CircuitGraphLabelAnchor, f32, egui::Rect, String)> {
    let mut candidates = Vec::new();
    let inset = 10.0;
    let gap = 6.0;
    let lane_gap = 4.0;
    let lane_step = 18.0 + lane_gap;
    let max_width = group
        .rect
        .width()
        .max(104.0)
        .min(graph_rect.width() * 0.35)
        .max(40.0);
    let label = truncate_group_label(
        &group.label,
        ((max_width - 12.0) / 6.6).floor().max(1.0) as usize,
    );
    let badge_size = vec2(
        (label.chars().count() as f32 * 6.6 + 12.0).min(max_width),
        18.0,
    );

    for lane in 0..6 {
        let lane_offset = lane as f32 * lane_step;
        let inside_top =
            (group.rect.top() + gap + lane_offset).min(group.rect.bottom() - badge_size.y - gap);
        let below_top = group.rect.bottom() + gap + lane_offset;
        let above_top = group.rect.top() - badge_size.y - gap - lane_offset;
        let side_top =
            (group.rect.top() + gap + lane_offset).min(group.rect.bottom() - badge_size.y - gap);

        candidates.push((
            CircuitGraphLabelAnchor::InsideTopLeft,
            lane as f32 * 2.0,
            egui::Rect::from_min_size(pos2(group.rect.left() + inset, inside_top), badge_size),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::InsideTopRight,
            lane as f32 * 2.0,
            egui::Rect::from_min_size(
                pos2(group.rect.right() - inset - badge_size.x, inside_top),
                badge_size,
            ),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::AboveLeft,
            10.0 + lane as f32 * 1.5,
            egui::Rect::from_min_size(pos2(group.rect.left() + inset, above_top), badge_size),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::AboveRight,
            10.0 + lane as f32 * 1.5,
            egui::Rect::from_min_size(
                pos2(group.rect.right() - inset - badge_size.x, above_top),
                badge_size,
            ),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::BelowLeft,
            12.0 + lane as f32 * 1.5,
            egui::Rect::from_min_size(pos2(group.rect.left() + inset, below_top), badge_size),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::BelowRight,
            12.0 + lane as f32 * 1.5,
            egui::Rect::from_min_size(
                pos2(group.rect.right() - inset - badge_size.x, below_top),
                badge_size,
            ),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::OutsideRight,
            16.0 + lane as f32,
            egui::Rect::from_min_size(pos2(group.rect.right() + gap, side_top), badge_size),
            label.clone(),
        ));
        candidates.push((
            CircuitGraphLabelAnchor::OutsideLeft,
            16.0 + lane as f32,
            egui::Rect::from_min_size(
                pos2(group.rect.left() - gap - badge_size.x, side_top),
                badge_size,
            ),
            label.clone(),
        ));
    }

    candidates
}

fn circuit_graph_gate_obstacles(
    gate_positions: &[egui::Pos2],
    gate_rect_template: egui::Rect,
) -> Vec<egui::Rect> {
    let mut obstacles = Vec::new();
    for (index, position) in gate_positions.iter().enumerate() {
        let gate_rect = egui::Rect::from_center_size(*position, gate_rect_template.size());
        obstacles.push(gate_rect);
        let gate_label_width = ((format!("g{index}").chars().count() as f32) * 6.4).max(18.0);
        obstacles.push(egui::Rect::from_center_size(
            pos2(position.x, position.y - 16.0 - 6.0),
            vec2(gate_label_width, 12.0),
        ));
    }
    obstacles
}

fn evaluate_circuit_graph_label(
    group_id: usize,
    candidate_rect: &egui::Rect,
    anchor_cost: f32,
    placements: &HashMap<usize, CircuitGraphLabelPlacement>,
    gate_obstacles: &[egui::Rect],
    graph_rect: egui::Rect,
) -> CircuitGraphLayoutEvaluation {
    let mut evaluation = CircuitGraphLayoutEvaluation {
        anchor_cost,
        ..CircuitGraphLayoutEvaluation::default()
    };

    for (other_group_id, placement) in placements {
        if *other_group_id == group_id {
            continue;
        }
        evaluation.label_overlap += rect_overlap_area(*candidate_rect, placement.rect);
        evaluation.label_proximity +=
            rect_overlap_area(candidate_rect.expand(6.0), placement.rect.expand(6.0));
    }

    for obstacle in gate_obstacles {
        evaluation.gate_overlap += rect_overlap_area(*candidate_rect, *obstacle);
        evaluation.gate_proximity +=
            rect_overlap_area(candidate_rect.expand(5.0), obstacle.expand(5.0));
    }

    evaluation.out_of_bounds = rect_out_of_bounds_area(*candidate_rect, graph_rect);
    evaluation.total = evaluation.label_overlap * 400.0
        + evaluation.label_proximity * 24.0
        + evaluation.gate_overlap * 120.0
        + evaluation.gate_proximity * 8.0
        + evaluation.out_of_bounds * 260.0
        + evaluation.anchor_cost;
    evaluation
}

fn default_circuit_graph_labels(
    root: &CircuitGraphGroupLayout,
    graph_rect: egui::Rect,
) -> HashMap<usize, CircuitGraphLabelPlacement> {
    let mut groups = Vec::new();
    collect_visible_circuit_graph_group_layouts(root, &mut groups);
    let mut placements = HashMap::new();
    for group in groups {
        if let Some((anchor, _, rect, label)) = circuit_graph_label_candidates(group, graph_rect)
            .into_iter()
            .next()
        {
            placements.insert(
                group.id,
                CircuitGraphLabelPlacement {
                    label,
                    rect,
                    anchor,
                },
            );
        }
    }
    placements
}

fn evaluate_circuit_graph_layout(
    root: &CircuitGraphGroupLayout,
    placements: &HashMap<usize, CircuitGraphLabelPlacement>,
    gate_positions: &[egui::Pos2],
    graph_rect: egui::Rect,
    gate_rect_template: egui::Rect,
) -> CircuitGraphLayoutEvaluation {
    let gate_obstacles = circuit_graph_gate_obstacles(gate_positions, gate_rect_template);
    let mut groups = Vec::new();
    collect_visible_circuit_graph_group_layouts(root, &mut groups);
    let mut evaluation = CircuitGraphLayoutEvaluation::default();

    for group in &groups {
        if let Some(placement) = placements.get(&group.id) {
            for obstacle in &gate_obstacles {
                evaluation.gate_overlap += rect_overlap_area(placement.rect, *obstacle);
                evaluation.gate_proximity +=
                    rect_overlap_area(placement.rect.expand(5.0), obstacle.expand(5.0));
            }
            evaluation.out_of_bounds += rect_out_of_bounds_area(placement.rect, graph_rect);
            evaluation.anchor_cost += match placement.anchor {
                CircuitGraphLabelAnchor::InsideTopLeft
                | CircuitGraphLabelAnchor::InsideTopRight => 0.0,
                CircuitGraphLabelAnchor::AboveLeft | CircuitGraphLabelAnchor::AboveRight => 10.0,
                CircuitGraphLabelAnchor::BelowLeft | CircuitGraphLabelAnchor::BelowRight => 12.0,
                CircuitGraphLabelAnchor::OutsideLeft | CircuitGraphLabelAnchor::OutsideRight => {
                    16.0
                }
            };
        }
    }

    for (index, left_group) in groups.iter().enumerate() {
        let Some(left) = placements.get(&left_group.id) else {
            continue;
        };
        for right_group in groups.iter().skip(index + 1) {
            let Some(right) = placements.get(&right_group.id) else {
                continue;
            };
            evaluation.label_overlap += rect_overlap_area(left.rect, right.rect);
            evaluation.label_proximity +=
                rect_overlap_area(left.rect.expand(6.0), right.rect.expand(6.0));
        }
    }

    evaluation.total = evaluation.label_overlap * 400.0
        + evaluation.label_proximity * 24.0
        + evaluation.gate_overlap * 120.0
        + evaluation.gate_proximity * 8.0
        + evaluation.out_of_bounds * 260.0
        + evaluation.anchor_cost;
    evaluation
}

fn rect_overlap_area(left: egui::Rect, right: egui::Rect) -> f32 {
    let width = (left.right().min(right.right()) - left.left().max(right.left())).max(0.0);
    let height = (left.bottom().min(right.bottom()) - left.top().max(right.top())).max(0.0);
    width * height
}

fn rect_out_of_bounds_area(rect: egui::Rect, bounds: egui::Rect) -> f32 {
    let mut area = 0.0;
    if rect.left() < bounds.left() {
        area += (bounds.left() - rect.left()) * rect.height();
    }
    if rect.right() > bounds.right() {
        area += (rect.right() - bounds.right()) * rect.height();
    }
    if rect.top() < bounds.top() {
        area += (bounds.top() - rect.top()) * rect.width();
    }
    if rect.bottom() > bounds.bottom() {
        area += (rect.bottom() - bounds.bottom()) * rect.width();
    }
    area
}

fn draw_circuit_group(
    painter: &egui::Painter,
    group: &CircuitGraphGroupLayout,
    label_layouts: &HashMap<usize, CircuitGraphLabelPlacement>,
) {
    if group.depth > 0 {
        let fill = circuit_group_fill(group.depth);
        let stroke = Stroke::new(1.0, circuit_group_stroke(group.depth));
        painter.rect_filled(group.rect, 8.0, fill);
        painter.rect_stroke(group.rect, 8.0, stroke, egui::StrokeKind::Inside);
    }

    for child in &group.children {
        draw_circuit_group(painter, child, label_layouts);
    }

    if group.depth > 0 {
        if let Some(label) = label_layouts.get(&group.id) {
            draw_circuit_group_label(painter, label, group.depth);
        }
    }
}

fn draw_circuit_group_label(
    painter: &egui::Painter,
    placement: &CircuitGraphLabelPlacement,
    depth: usize,
) {
    painter.rect_filled(placement.rect, 6.0, circuit_group_badge_fill(depth));
    painter.rect_stroke(
        placement.rect,
        6.0,
        Stroke::new(1.0, circuit_group_stroke(depth)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        placement.rect.center(),
        Align2::CENTER_CENTER,
        &placement.label,
        FontId::monospace(11.0),
        Color32::from_gray(240),
    );
}

fn truncate_group_label(label: &str, max_chars: usize) -> String {
    let count = label.chars().count();
    if count <= max_chars {
        return label.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let mut truncated = label.chars().take(max_chars - 1).collect::<String>();
    truncated.push('…');
    truncated
}

fn circuit_group_fill(depth: usize) -> Color32 {
    match depth % 4 {
        0 => Color32::from_rgba_unmultiplied(30, 44, 58, 28),
        1 => Color32::from_rgba_unmultiplied(44, 55, 32, 24),
        2 => Color32::from_rgba_unmultiplied(52, 40, 28, 24),
        _ => Color32::from_rgba_unmultiplied(44, 32, 54, 24),
    }
}

fn circuit_group_stroke(depth: usize) -> Color32 {
    match depth % 4 {
        0 => Color32::from_rgb(86, 126, 164),
        1 => Color32::from_rgb(118, 154, 92),
        2 => Color32::from_rgb(173, 126, 88),
        _ => Color32::from_rgb(140, 110, 176),
    }
}

fn circuit_group_badge_fill(depth: usize) -> Color32 {
    match depth % 4 {
        0 => Color32::from_rgb(26, 42, 56),
        1 => Color32::from_rgb(40, 52, 28),
        2 => Color32::from_rgb(52, 36, 24),
        _ => Color32::from_rgb(42, 30, 52),
    }
}

fn draw_signal_edge(painter: &egui::Painter, source: egui::Pos2, target: egui::Pos2, active: bool) {
    let stroke = Stroke::new(2.0, edge_color(active));
    let curve = signal_curve_points(source, target);
    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        curve,
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
}

fn signal_curve_points(source: egui::Pos2, target: egui::Pos2) -> [egui::Pos2; 4] {
    let dx = target.x - source.x;
    if dx >= 0.0 {
        let handle = dx * 0.45;
        [
            source,
            pos2(source.x + handle, source.y),
            pos2(target.x - handle, target.y),
            target,
        ]
    } else {
        let loop_height = 28.0 + dx.abs() * 0.25;
        [
            source,
            pos2(source.x + 36.0, source.y - loop_height),
            pos2(target.x - 36.0, target.y - loop_height),
            target,
        ]
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_circuit_graph_png_native(
    products: &IntermediateProducts,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
    width: u32,
    height: u32,
    output_path: &str,
) -> Result<(), String> {
    let scene = build_circuit_graph_render_scene(
        products,
        simulation,
        compiled,
        egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(width as f32, height as f32)),
    );
    let font = export_font()?;
    let mut image = RgbaImage::from_pixel(width, height, rgba_from_color(Color32::from_gray(18)));
    raster_stroke_rect(&mut image, scene.rect.shrink(0.5), Color32::from_gray(70));

    if let Some(layout) = &scene.group_layout {
        draw_circuit_group_raster(&mut image, &font, layout, &scene.label_layouts)?;
    }

    let gate_count = compiled.gates_sequential.len();
    let input_x = scene.rect.left() + CIRCUIT_GRAPH_HORIZONTAL_PADDING + 56.0;
    let output_x = scene.rect.right() - CIRCUIT_GRAPH_HORIZONTAL_PADDING - 56.0;

    for (gate_index, gate) in compiled.gates_sequential.iter().enumerate() {
        let gate_pos = scene
            .gate_positions
            .get(gate_index)
            .copied()
            .unwrap_or(scene.rect.center());
        for input in [&gate.0, &gate.1] {
            let (source_pos, active) = gate_source_position(
                input,
                &scene.input_positions,
                &scene.gate_positions,
                input_x,
                output_x,
                &simulation.inputs,
                &scene.gate_states,
            );
            raster_draw_signal_edge(&mut image, source_pos, gate_pos, active);
        }
    }

    for (output_index, source) in compiled.outputs.iter().enumerate() {
        if let Some(target) = scene.output_positions.get(output_index).copied() {
            let (source_pos, active) = output_source_position(
                *source,
                gate_count,
                &scene.input_positions,
                &scene.gate_positions,
                input_x,
                output_x,
                &simulation.inputs,
                &scene.gate_states,
            );
            raster_draw_signal_edge(&mut image, source_pos, target, active);
        }
    }

    for (index, position) in scene.input_positions.iter().enumerate() {
        let active = simulation.inputs.get(index).copied().unwrap_or(false);
        raster_fill_circle(
            &mut image,
            *position,
            10.0,
            node_color(active, NodeKind::Input),
        );
        raster_draw_text(
            &mut image,
            &font,
            &simulation
                .input_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("in{index}")),
            pos2(position.x - 16.0, position.y),
            11.0,
            "right",
            "middle",
            Color32::WHITE,
        );
    }

    for (index, position) in scene.gate_positions.iter().enumerate() {
        let active = scene.gate_states.get(index).copied().unwrap_or(false);
        let gate_rect = egui::Rect::from_center_size(*position, vec2(36.0, 20.0));
        raster_fill_rect(&mut image, gate_rect, node_color(active, NodeKind::Gate));
        raster_stroke_rect(&mut image, gate_rect, Color32::from_gray(90));
        raster_draw_text(
            &mut image,
            &font,
            "nor",
            gate_rect.center(),
            10.0,
            "center",
            "middle",
            Color32::WHITE,
        );
        raster_draw_text(
            &mut image,
            &font,
            &format!("g{index}"),
            pos2(position.x, position.y - 16.0),
            10.0,
            "center",
            "bottom",
            Color32::GRAY,
        );
    }

    for (index, position) in scene.output_positions.iter().enumerate() {
        let active = simulation.outputs.get(index).copied().unwrap_or(false);
        raster_fill_rect(
            &mut image,
            egui::Rect::from_center_size(*position, vec2(20.0, 20.0)),
            node_color(active, NodeKind::Output),
        );
        raster_draw_text(
            &mut image,
            &font,
            &simulation
                .output_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("out{index}")),
            pos2(position.x + 16.0, position.y),
            11.0,
            "left",
            "middle",
            Color32::WHITE,
        );
    }

    image
        .save(output_path)
        .map_err(|error| format!("failed to save PNG: {error}"))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn export_font() -> Result<FontArc, String> {
    let font_defs = egui::FontDefinitions::default();
    let font_data = font_defs
        .font_data
        .get("Hack")
        .or_else(|| font_defs.font_data.values().next())
        .ok_or_else(|| "default font data unavailable".to_string())?;
    FontArc::try_from_vec(font_data.font.as_ref().to_vec())
        .map_err(|error| format!("failed to load export font: {error}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn draw_circuit_group_raster(
    image: &mut RgbaImage,
    font: &FontArc,
    group: &CircuitGraphGroupLayout,
    label_layouts: &HashMap<usize, CircuitGraphLabelPlacement>,
) -> Result<(), String> {
    if group.depth > 0 {
        raster_fill_rect(image, group.rect, circuit_group_fill(group.depth));
        raster_stroke_rect(image, group.rect, circuit_group_stroke(group.depth));
    }

    for child in &group.children {
        draw_circuit_group_raster(image, font, child, label_layouts)?;
    }

    if group.depth > 0 {
        if let Some(label) = label_layouts.get(&group.id) {
            raster_fill_rect(image, label.rect, circuit_group_badge_fill(group.depth));
            raster_stroke_rect(image, label.rect, circuit_group_stroke(group.depth));
            raster_draw_text(
                image,
                font,
                &label.label,
                label.rect.center(),
                11.0,
                "center",
                "middle",
                Color32::from_gray(240),
            );
        }
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_draw_signal_edge(image: &mut RgbaImage, source: egui::Pos2, target: egui::Pos2, active: bool) {
    let curve = signal_curve_points(source, target);
    let mut previous = curve[0];
    let steps = 96;
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let current = cubic_curve_point(curve, t);
        raster_draw_thick_segment(image, previous, current, 1.25, edge_color(active));
        previous = current;
    }
}

fn cubic_curve_point(curve: [egui::Pos2; 4], t: f32) -> egui::Pos2 {
    let one_minus_t = 1.0 - t;
    let p0 = one_minus_t.powi(3);
    let p1 = 3.0 * one_minus_t.powi(2) * t;
    let p2 = 3.0 * one_minus_t * t.powi(2);
    let p3 = t.powi(3);
    pos2(
        curve[0].x * p0 + curve[1].x * p1 + curve[2].x * p2 + curve[3].x * p3,
        curve[0].y * p0 + curve[1].y * p1 + curve[2].y * p2 + curve[3].y * p3,
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_draw_thick_segment(
    image: &mut RgbaImage,
    start: egui::Pos2,
    end: egui::Pos2,
    radius: f32,
    color: Color32,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let point = pos2(start.x + dx * t, start.y + dy * t);
        raster_fill_circle(image, point, radius, color);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_fill_rect(image: &mut RgbaImage, rect: egui::Rect, color: Color32) {
    let min_x = rect.left().floor().max(0.0) as i32;
    let max_x = rect.right().ceil().min(image.width() as f32) as i32;
    let min_y = rect.top().floor().max(0.0) as i32;
    let max_y = rect.bottom().ceil().min(image.height() as f32) as i32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            blend_pixel(image, x, y, color);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_stroke_rect(image: &mut RgbaImage, rect: egui::Rect, color: Color32) {
    let left = rect.left().round() as i32;
    let right = rect.right().round() as i32;
    let top = rect.top().round() as i32;
    let bottom = rect.bottom().round() as i32;
    for x in left..=right {
        blend_pixel(image, x, top, color);
        blend_pixel(image, x, bottom, color);
    }
    for y in top..=bottom {
        blend_pixel(image, left, y, color);
        blend_pixel(image, right, y, color);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_fill_circle(image: &mut RgbaImage, center: egui::Pos2, radius: f32, color: Color32) {
    let min_x = (center.x - radius).floor() as i32;
    let max_x = (center.x + radius).ceil() as i32;
    let min_y = (center.y - radius).floor() as i32;
    let max_y = (center.y + radius).ceil() as i32;
    let radius_sq = radius * radius;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            if dx * dx + dy * dy <= radius_sq {
                blend_pixel(image, x, y, color);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn raster_draw_text(
    image: &mut RgbaImage,
    font: &FontArc,
    text: &str,
    position: egui::Pos2,
    size: f32,
    align: &str,
    baseline: &str,
    color: Color32,
) {
    let scale = PxScale::from(size);
    let scaled = font.as_scaled(scale);
    let width = text
        .chars()
        .map(|ch| scaled.h_advance(scaled.glyph_id(ch)))
        .sum::<f32>();
    let ascent = scaled.ascent();
    let descent = scaled.descent();
    let start_x = match align {
        "center" => position.x - width * 0.5,
        "right" => position.x - width,
        _ => position.x,
    };
    let baseline_y = match baseline {
        "middle" => position.y + (ascent + descent) * 0.5,
        "bottom" => position.y + descent,
        "top" => position.y + ascent,
        _ => position.y,
    };

    let mut caret_x = start_x;
    for ch in text.chars() {
        let glyph_id = scaled.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, point(caret_x, baseline_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                let px = bounds.min.x as i32 + x as i32;
                let py = bounds.min.y as i32 + y as i32;
                let alpha = (color.a() as f32 * coverage).round().clamp(0.0, 255.0) as u8;
                blend_pixel(
                    image,
                    px,
                    py,
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha),
                );
            });
        }
        caret_x += scaled.h_advance(glyph_id);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn blend_pixel(image: &mut RgbaImage, x: i32, y: i32, color: Color32) {
    if x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 {
        return;
    }
    let dest = image.get_pixel_mut(x as u32, y as u32);
    let src_alpha = color.a() as f32 / 255.0;
    let dst_alpha = dest[3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    let blend_channel = |src: u8, dst: u8| -> u8 {
        if out_alpha <= 0.0 {
            0
        } else {
            (((src as f32 * src_alpha) + (dst as f32 * dst_alpha * (1.0 - src_alpha))) / out_alpha)
                .round()
                .clamp(0.0, 255.0) as u8
        }
    };
    *dest = Rgba([
        blend_channel(color.r(), dest[0]),
        blend_channel(color.g(), dest[1]),
        blend_channel(color.b(), dest[2]),
        (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    ]);
}

#[cfg(not(target_arch = "wasm32"))]
fn rgba_from_color(color: Color32) -> Rgba<u8> {
    Rgba([color.r(), color.g(), color.b(), color.a()])
}

#[cfg(target_arch = "wasm32")]
fn export_circuit_graph_png(
    products: &IntermediateProducts,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window not available".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "document not available".to_string())?;
    let canvas = document
        .create_element("canvas")
        .map_err(|error| format!("failed to create canvas: {error:?}"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "canvas element type mismatch".to_string())?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas
        .get_context("2d")
        .map_err(|error| format!("failed to get canvas context: {error:?}"))?
        .ok_or_else(|| "2d canvas context not available".to_string())?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "2d canvas context type mismatch".to_string())?;

    let scene = build_circuit_graph_render_scene(
        products,
        simulation,
        compiled,
        egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(width as f32, height as f32)),
    );
    render_circuit_graph_to_canvas(&context, simulation, compiled, &scene)?;

    let anchor = document
        .create_element("a")
        .map_err(|error| format!("failed to create download link: {error:?}"))?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "anchor element type mismatch".to_string())?;
    anchor.set_href(
        &canvas
            .to_data_url_with_type("image/png")
            .map_err(|error| format!("failed to encode PNG: {error:?}"))?,
    );
    anchor.set_download(&format!(
        "{}-circuit-{}x{}.png",
        simulation.module_name, width, height
    ));
    anchor.click();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn render_circuit_graph_to_canvas(
    context: &web_sys::CanvasRenderingContext2d,
    simulation: &SimulationState,
    compiled: &compiler::types::CompiledModule,
    scene: &CircuitGraphRenderScene,
) -> Result<(), String> {
    context.set_fill_style_str(&canvas_color(Color32::from_gray(18)));
    context.fill_rect(
        scene.rect.left() as f64,
        scene.rect.top() as f64,
        scene.rect.width() as f64,
        scene.rect.height() as f64,
    );
    canvas_fill_and_stroke_round_rect(
        context,
        scene.rect.shrink(0.5),
        6.0,
        None,
        Some((1.0, Color32::from_gray(70))),
    )?;

    if let Some(layout) = &scene.group_layout {
        draw_circuit_group_canvas(context, layout, &scene.label_layouts)?;
    }

    let gate_count = compiled.gates_sequential.len();
    let input_x = scene.rect.left() + CIRCUIT_GRAPH_HORIZONTAL_PADDING + 56.0;
    let output_x = scene.rect.right() - CIRCUIT_GRAPH_HORIZONTAL_PADDING - 56.0;

    for (gate_index, gate) in compiled.gates_sequential.iter().enumerate() {
        let gate_pos = scene
            .gate_positions
            .get(gate_index)
            .copied()
            .unwrap_or(scene.rect.center());
        for input in [&gate.0, &gate.1] {
            let (source_pos, active) = gate_source_position(
                input,
                &scene.input_positions,
                &scene.gate_positions,
                input_x,
                output_x,
                &simulation.inputs,
                &scene.gate_states,
            );
            draw_signal_edge_canvas(context, source_pos, gate_pos, active)?;
        }
    }

    for (output_index, source) in compiled.outputs.iter().enumerate() {
        if let Some(target) = scene.output_positions.get(output_index).copied() {
            let (source_pos, active) = output_source_position(
                *source,
                gate_count,
                &scene.input_positions,
                &scene.gate_positions,
                input_x,
                output_x,
                &simulation.inputs,
                &scene.gate_states,
            );
            draw_signal_edge_canvas(context, source_pos, target, active)?;
        }
    }

    for (index, position) in scene.input_positions.iter().enumerate() {
        let active = simulation.inputs.get(index).copied().unwrap_or(false);
        context.begin_path();
        context.set_fill_style_str(&canvas_color(node_color(active, NodeKind::Input)));
        context
            .arc(
                position.x as f64,
                position.y as f64,
                10.0,
                0.0,
                std::f64::consts::TAU,
            )
            .map_err(|error| format!("failed to draw input node: {error:?}"))?;
        context.fill();
        canvas_draw_text(
            context,
            &simulation
                .input_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("in{index}")),
            pos2(position.x - 16.0, position.y),
            "11px monospace",
            "right",
            "middle",
            Color32::WHITE,
        )?;
    }

    for (index, position) in scene.gate_positions.iter().enumerate() {
        let active = scene.gate_states.get(index).copied().unwrap_or(false);
        let gate_rect = egui::Rect::from_center_size(*position, vec2(36.0, 20.0));
        canvas_fill_and_stroke_round_rect(
            context,
            gate_rect,
            4.0,
            Some(node_color(active, NodeKind::Gate)),
            Some((1.0, Color32::from_gray(90))),
        )?;
        canvas_draw_text(
            context,
            "nor",
            gate_rect.center(),
            "10px monospace",
            "center",
            "middle",
            Color32::WHITE,
        )?;
        canvas_draw_text(
            context,
            &format!("g{index}"),
            pos2(position.x, position.y - 16.0),
            "10px monospace",
            "center",
            "bottom",
            Color32::GRAY,
        )?;
    }

    for (index, position) in scene.output_positions.iter().enumerate() {
        let active = simulation.outputs.get(index).copied().unwrap_or(false);
        canvas_fill_and_stroke_round_rect(
            context,
            egui::Rect::from_center_size(*position, vec2(20.0, 20.0)),
            3.0,
            Some(node_color(active, NodeKind::Output)),
            None,
        )?;
        canvas_draw_text(
            context,
            &simulation
                .output_labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("out{index}")),
            pos2(position.x + 16.0, position.y),
            "11px monospace",
            "left",
            "middle",
            Color32::WHITE,
        )?;
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn draw_circuit_group_canvas(
    context: &web_sys::CanvasRenderingContext2d,
    group: &CircuitGraphGroupLayout,
    label_layouts: &HashMap<usize, CircuitGraphLabelPlacement>,
) -> Result<(), String> {
    if group.depth > 0 {
        canvas_fill_and_stroke_round_rect(
            context,
            group.rect,
            8.0,
            Some(circuit_group_fill(group.depth)),
            Some((1.0, circuit_group_stroke(group.depth))),
        )?;
    }

    for child in &group.children {
        draw_circuit_group_canvas(context, child, label_layouts)?;
    }

    if group.depth > 0 {
        if let Some(label) = label_layouts.get(&group.id) {
            canvas_fill_and_stroke_round_rect(
                context,
                label.rect,
                6.0,
                Some(circuit_group_badge_fill(group.depth)),
                Some((1.0, circuit_group_stroke(group.depth))),
            )?;
            canvas_draw_text(
                context,
                &label.label,
                label.rect.center(),
                "11px monospace",
                "center",
                "middle",
                Color32::from_gray(240),
            )?;
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn draw_signal_edge_canvas(
    context: &web_sys::CanvasRenderingContext2d,
    source: egui::Pos2,
    target: egui::Pos2,
    active: bool,
) -> Result<(), String> {
    let curve = signal_curve_points(source, target);
    context.begin_path();
    context.set_line_width(2.0);
    context.set_stroke_style_str(&canvas_color(edge_color(active)));
    context.move_to(curve[0].x as f64, curve[0].y as f64);
    context.bezier_curve_to(
        curve[1].x as f64,
        curve[1].y as f64,
        curve[2].x as f64,
        curve[2].y as f64,
        curve[3].x as f64,
        curve[3].y as f64,
    );
    context.stroke();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn canvas_fill_and_stroke_round_rect(
    context: &web_sys::CanvasRenderingContext2d,
    rect: egui::Rect,
    radius: f32,
    fill: Option<Color32>,
    stroke: Option<(f64, Color32)>,
) -> Result<(), String> {
    context.begin_path();
    canvas_round_rect_path(context, rect, radius)?;
    if let Some(fill) = fill {
        context.set_fill_style_str(&canvas_color(fill));
        context.fill();
    }
    if let Some((width, stroke_color)) = stroke {
        context.set_line_width(width);
        context.set_stroke_style_str(&canvas_color(stroke_color));
        context.stroke();
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn canvas_round_rect_path(
    context: &web_sys::CanvasRenderingContext2d,
    rect: egui::Rect,
    radius: f32,
) -> Result<(), String> {
    let radius = radius.min(rect.width() * 0.5).min(rect.height() * 0.5) as f64;
    let left = rect.left() as f64;
    let right = rect.right() as f64;
    let top = rect.top() as f64;
    let bottom = rect.bottom() as f64;

    context.move_to(left + radius, top);
    context.line_to(right - radius, top);
    context.quadratic_curve_to(right, top, right, top + radius);
    context.line_to(right, bottom - radius);
    context.quadratic_curve_to(right, bottom, right - radius, bottom);
    context.line_to(left + radius, bottom);
    context.quadratic_curve_to(left, bottom, left, bottom - radius);
    context.line_to(left, top + radius);
    context.quadratic_curve_to(left, top, left + radius, top);
    context.close_path();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn canvas_draw_text(
    context: &web_sys::CanvasRenderingContext2d,
    text: &str,
    position: egui::Pos2,
    font: &str,
    align: &str,
    baseline: &str,
    color: Color32,
) -> Result<(), String> {
    context.set_font(font);
    context.set_text_align(align);
    context.set_text_baseline(baseline);
    context.set_fill_style_str(&canvas_color(color));
    context
        .fill_text(text, position.x as f64, position.y as f64)
        .map_err(|error| format!("failed to draw text: {error:?}"))
}

#[cfg(target_arch = "wasm32")]
fn canvas_color(color: Color32) -> String {
    format!(
        "rgba({}, {}, {}, {:.3})",
        color.r(),
        color.g(),
        color.b(),
        color.a() as f32 / 255.0
    )
}

fn vertical_positions(count: usize, rect: &egui::Rect) -> Vec<egui::Pos2> {
    if count == 0 {
        return Vec::new();
    }
    let top = rect.top() + 24.0;
    let bottom = rect.bottom() - 24.0;
    if count == 1 {
        return vec![pos2(0.0, (top + bottom) * 0.5)];
    }
    (0..count)
        .map(|index| {
            let t = index as f32 / (count - 1) as f32;
            pos2(0.0, egui::lerp(top..=bottom, t))
        })
        .collect()
}

fn gate_source_position(
    input: &CompiledGateInput,
    input_positions: &[egui::Pos2],
    gate_positions: &[egui::Pos2],
    input_x: f32,
    gate_exit_x: f32,
    input_states: &[bool],
    gate_states: &[bool],
) -> (egui::Pos2, bool) {
    match input {
        CompiledGateInput::Input(index) => {
            let index = *index as usize;
            let mut position = input_positions
                .get(index)
                .copied()
                .unwrap_or(pos2(input_x, 0.0));
            position.x = input_x;
            (position, input_states.get(index).copied().unwrap_or(false))
        }
        CompiledGateInput::NorGate(index) => {
            let index = *index as usize;
            let mut position = gate_positions
                .get(index)
                .copied()
                .unwrap_or(pos2(gate_exit_x, 0.0));
            position.x += 18.0;
            (position, gate_states.get(index).copied().unwrap_or(false))
        }
    }
}

fn output_source_position(
    source: u32,
    gate_count: usize,
    input_positions: &[egui::Pos2],
    gate_positions: &[egui::Pos2],
    input_x: f32,
    output_x: f32,
    input_states: &[bool],
    gate_states: &[bool],
) -> (egui::Pos2, bool) {
    let index = source as usize;
    if index < gate_count {
        let mut position = gate_positions
            .get(index)
            .copied()
            .unwrap_or(pos2(output_x, 0.0));
        position.x += 18.0;
        (position, gate_states.get(index).copied().unwrap_or(false))
    } else {
        let input_index = index.saturating_sub(gate_count);
        let mut position = input_positions
            .get(input_index)
            .copied()
            .unwrap_or(pos2(input_x, 0.0));
        position.x = input_x;
        (
            position,
            input_states.get(input_index).copied().unwrap_or(false),
        )
    }
}

fn format_bits(bits: &[bool]) -> String {
    if bits.is_empty() {
        "-".to_string()
    } else {
        bits.iter()
            .map(|value| if *value { "t" } else { "f" })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn rgb_tuple((r, g, b): (u8, u8, u8)) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn edge_color(active: bool) -> Color32 {
    if active {
        Color32::from_rgb(120, 210, 120)
    } else {
        Color32::from_gray(70)
    }
}

enum NodeKind {
    Input,
    Gate,
    Output,
}

fn node_color(active: bool, kind: NodeKind) -> Color32 {
    match (active, kind) {
        (true, NodeKind::Input) => Color32::from_rgb(70, 170, 255),
        (false, NodeKind::Input) => Color32::from_rgb(20, 70, 110),
        (true, NodeKind::Gate) => Color32::from_rgb(80, 210, 120),
        (false, NodeKind::Gate) => Color32::from_rgb(40, 70, 45),
        (true, NodeKind::Output) => Color32::from_rgb(255, 180, 90),
        (false, NodeKind::Output) => Color32::from_rgb(110, 70, 30),
    }
}

enum DiagnosticLevel {
    Error,
    Warn,
}

fn diagnostic_label(ui: &mut egui::Ui, message: &str, level: DiagnosticLevel) {
    let (fill, stroke, text) = match level {
        DiagnosticLevel::Error => (
            Color32::from_rgb(44, 12, 14),
            Stroke::new(1.0, Color32::from_rgb(150, 50, 55)),
            Color32::from_rgb(255, 180, 180),
        ),
        DiagnosticLevel::Warn => (
            Color32::from_rgb(52, 42, 14),
            Stroke::new(1.0, Color32::from_rgb(160, 120, 45)),
            Color32::from_rgb(255, 230, 170),
        ),
    };
    egui::Frame::NONE
        .fill(fill)
        .stroke(stroke)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            ui.label(RichText::new(message).color(text));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(
        id: usize,
        label: &str,
        depth: usize,
        min: (f32, f32),
        max: (f32, f32),
        children: Vec<CircuitGraphGroupLayout>,
    ) -> CircuitGraphGroupLayout {
        CircuitGraphGroupLayout {
            id,
            label: label.to_string(),
            depth,
            rect: egui::Rect::from_min_max(pos2(min.0, min.1), pos2(max.0, max.1)),
            children,
        }
    }

    fn gate_template() -> egui::Rect {
        egui::Rect::from_center_size(pos2(0.0, 0.0), vec2(44.0, 24.0))
    }

    #[test]
    fn circuit_graph_layout_optimizer_beats_default_on_nested_groups() {
        let layout = group(
            0,
            "counter_swc",
            0,
            (20.0, 20.0),
            (360.0, 320.0),
            vec![group(
                1,
                "8inc",
                1,
                (40.0, 40.0),
                (280.0, 260.0),
                vec![
                    group(2, "hadd0", 2, (52.0, 56.0), (160.0, 132.0), vec![]),
                    group(3, "hadd1", 2, (58.0, 74.0), (172.0, 154.0), vec![]),
                    group(4, "and", 2, (110.0, 96.0), (226.0, 182.0), vec![]),
                    group(5, "not", 2, (126.0, 116.0), (238.0, 198.0), vec![]),
                ],
            )],
        );
        let graph_rect = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(420.0, 360.0));
        let gate_positions = vec![
            pos2(84.0, 68.0),
            pos2(96.0, 92.0),
            pos2(116.0, 116.0),
            pos2(128.0, 140.0),
            pos2(142.0, 164.0),
            pos2(156.0, 188.0),
        ];

        let baseline = default_circuit_graph_labels(&layout, graph_rect);
        let optimized =
            optimize_circuit_graph_labels(&layout, &gate_positions, graph_rect, gate_template());
        let baseline_eval = evaluate_circuit_graph_layout(
            &layout,
            &baseline,
            &gate_positions,
            graph_rect,
            gate_template(),
        );
        let optimized_eval = evaluate_circuit_graph_layout(
            &layout,
            &optimized,
            &gate_positions,
            graph_rect,
            gate_template(),
        );

        assert!(
            optimized_eval.total < baseline_eval.total,
            "optimized={optimized_eval:?} baseline={baseline_eval:?}"
        );
        assert!(
            optimized_eval.label_overlap < baseline_eval.label_overlap,
            "optimized={optimized_eval:?} baseline={baseline_eval:?}"
        );
    }

    #[test]
    fn circuit_graph_layout_optimizer_finds_non_overlapping_labels_when_space_exists() {
        let layout = group(
            0,
            "root_module",
            0,
            (20.0, 20.0),
            (420.0, 320.0),
            vec![
                group(1, "adder_lane_0", 1, (48.0, 48.0), (180.0, 136.0), vec![]),
                group(2, "adder_lane_1", 1, (48.0, 154.0), (180.0, 242.0), vec![]),
                group(3, "decoder", 1, (220.0, 48.0), (360.0, 136.0), vec![]),
                group(4, "selector", 1, (220.0, 154.0), (360.0, 242.0), vec![]),
            ],
        );
        let graph_rect = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(480.0, 360.0));
        let gate_positions = vec![
            pos2(92.0, 80.0),
            pos2(96.0, 190.0),
            pos2(272.0, 80.0),
            pos2(280.0, 190.0),
        ];

        let optimized =
            optimize_circuit_graph_labels(&layout, &gate_positions, graph_rect, gate_template());
        let optimized_eval = evaluate_circuit_graph_layout(
            &layout,
            &optimized,
            &gate_positions,
            graph_rect,
            gate_template(),
        );

        assert!(
            optimized_eval.label_overlap <= 0.1,
            "optimized={optimized_eval:?}"
        );
        assert!(
            optimized_eval.out_of_bounds <= 0.1,
            "optimized={optimized_eval:?}"
        );
    }

    #[test]
    fn circuit_graph_layout_optimizer_reduces_gate_overlap() {
        let layout = group(
            0,
            "dense_cluster",
            0,
            (16.0, 16.0),
            (220.0, 180.0),
            vec![group(1, "hotspot", 1, (28.0, 28.0), (170.0, 120.0), vec![])],
        );
        let graph_rect = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(280.0, 220.0));
        let gate_positions = vec![pos2(54.0, 42.0), pos2(88.0, 44.0), pos2(120.0, 46.0)];

        let baseline = default_circuit_graph_labels(&layout, graph_rect);
        let optimized =
            optimize_circuit_graph_labels(&layout, &gate_positions, graph_rect, gate_template());
        let baseline_eval = evaluate_circuit_graph_layout(
            &layout,
            &baseline,
            &gate_positions,
            graph_rect,
            gate_template(),
        );
        let optimized_eval = evaluate_circuit_graph_layout(
            &layout,
            &optimized,
            &gate_positions,
            graph_rect,
            gate_template(),
        );

        assert!(
            optimized_eval.gate_overlap < baseline_eval.gate_overlap,
            "optimized={optimized_eval:?} baseline={baseline_eval:?}"
        );
    }

    #[test]
    fn circuit_graph_gate_layout_optimizer_reduces_wire_crossings() {
        let compiled = compiler::types::CompiledModule {
            func: false,
            name: "crossing_demo".to_string(),
            inputs: 2,
            outputs: vec![2, 3],
            gates_sequential: vec![
                (CompiledGateInput::Input(0), CompiledGateInput::Input(0)),
                (CompiledGateInput::Input(1), CompiledGateInput::Input(1)),
                (CompiledGateInput::NorGate(1), CompiledGateInput::NorGate(1)),
                (CompiledGateInput::NorGate(0), CompiledGateInput::NorGate(0)),
            ],
            gates_symmetry: Vec::new(),
        };

        let baseline = default_circuit_graph_gate_layout(&compiled);
        let optimized = optimize_circuit_graph_gate_layout(&compiled, 2, 2);
        let baseline_eval = evaluate_circuit_graph_wires(&compiled, 2, 2, &baseline);
        let optimized_eval = evaluate_circuit_graph_wires(&compiled, 2, 2, &optimized);

        assert!(
            optimized_eval.crossings < baseline_eval.crossings,
            "optimized={optimized_eval:?} baseline={baseline_eval:?}"
        );
        assert!(
            optimized_eval.total < baseline_eval.total,
            "optimized={optimized_eval:?} baseline={baseline_eval:?}"
        );
    }

    #[test]
    fn circuit_graph_crossing_evaluator_detects_crossed_curves() {
        let curves = vec![
            circuit_graph_curve_points(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            circuit_graph_curve_points(pos2(0.0, 1.0), pos2(1.0, 0.0)),
        ];

        assert_eq!(circuit_graph_curve_crossings(&curves), 1.0);
    }
}
