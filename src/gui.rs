use eframe::{egui, Frame};
use egui::{CentralPanel, ScrollArea, TextEdit, TopBottomPanel};
use std::rc::Rc;
use std::cell::RefCell;
use crate::environment::Environment;
use crate::evaluator::{Evaluator, EvalError};
use crate::parse_input;

pub struct ComixApp {
    source: String,
    output: String,
    evaluator: Evaluator,
}

impl ComixApp {
    pub fn new(env: Rc<RefCell<Environment>>, source: String) -> Self {
        let evaluator = Evaluator::new(env, source);
        Self {
            source: String::new(),
            output: String::new(),
            evaluator,
        }
    }

    fn run_code(&mut self) {
        self.output.clear();
        self.evaluator.set_source(self.source.clone());

        match parse_input(&self.source) {
            Ok(stmts) => {
                match self.evaluator.evaluate(&stmts) {
                    Ok(_) => {
                        self.output.push_str("Execution completed.\n");
                        let mut var_str = String::new();
                        self.evaluator.dump_env_to_string(&mut var_str);
                        if !var_str.is_empty() {
                            self.output.push_str(&format!("Variables:\n{}", var_str));
                        }
                    }
                    Err(EvalError::Runtime(msg)) => {
                        self.output.push_str(&format!("Runtime error: {}\n", msg));
                    }
                    Err(EvalError::Return(v)) => {
                        self.output.push_str(&format!("Function returned: {}\n", v));
                    }
                }
            }
            Err(e) => {
                self.output.push_str(&format!("Parse error: {}\n", e));
            }
        }
    }
}

impl eframe::App for ComixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // 顶部面板
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Comix");
                ui.label("Language Interpreter");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Run").clicked() {
                        self.run_code();
                    }
                });
            });
        });

        // 底部面板：输出区域（固定位置，永不消失）
        TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Output");
                ScrollArea::vertical()
                    .id_source("output_scroll")
                    .max_height(600.0)   // 最大高度设为 600
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut self.output)
                                .font(egui::FontId::monospace(14.0))
                                .desired_rows(20)   // 期望显示 20 行
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
        });

        // 中央面板：代码编辑器（自动填满剩余空间，内容过多时内部滚动）
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Code Editor");
            ScrollArea::vertical()
                .id_source("code_editor_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.source)
                            .font(egui::FontId::monospace(14.0))
                            .desired_rows(20)
                            .desired_width(f32::INFINITY),
                    );
                });
        });
    }
}