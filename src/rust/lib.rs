mod compiler;
mod test;
mod transpiler;
mod vm;

#[cfg(feature = "web")]
mod resourcemanager;
#[cfg(feature = "web")]
mod web_bindings;

#[cfg(feature = "egui-web")]
mod egui_app;
