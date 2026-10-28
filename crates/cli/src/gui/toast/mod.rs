#![cfg(feature = "gui")]

use queue::{TOAST_LIFETIME, ToastNotice, ToastQueue};
use slint::ComponentHandle;
use slint::{Model, ModelRc, VecModel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{MainWindow, ToastItem, ToastState, ToastTiming};

mod queue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastLevel {
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn to_int(self) -> i32 {
        match self {
            ToastLevel::Success => 0,
            ToastLevel::Warning => 1,
            ToastLevel::Error => 2,
        }
    }
}

#[derive(Clone)]
pub struct Toaster {
    state: Arc<Mutex<ToastQueue>>,
    ui: slint::Weak<MainWindow>,
}

impl Toaster {
    pub fn install(ui: &MainWindow) -> Self {
        let state = Arc::new(Mutex::new(ToastQueue::default()));
        let callback_state = state.clone();
        let callback_weak = ui.as_weak();

        ui.global::<ToastState>()
            .on_close_requested(move |toast_id| {
                if let Some(ui) = callback_weak.upgrade() {
                    dismiss_toast(&ui, &callback_state, toast_id);
                }
            });

        ui.global::<ToastState>()
            .set_items(ModelRc::new(VecModel::<ToastItem>::default()));

        Self {
            state,
            ui: ui.as_weak(),
        }
    }

    pub fn show(&self, text: impl Into<String>, level: ToastLevel) {
        if let Some(ui) = self.ui.upgrade() {
            enqueue_toast(&ui, &self.state, text.into(), level.to_int());
        }
    }
}

fn exit_duration(ui: &MainWindow) -> Duration {
    Duration::from_millis(ui.global::<ToastTiming>().get_exit() as u64)
}

fn schedule_removal(ui: &MainWindow, state: Arc<Mutex<ToastQueue>>, id: u32) {
    let duration = exit_duration(ui);
    let weak = ui.as_weak();
    slint::Timer::single_shot(duration, move || {
        if let Some(ui) = weak.upgrade() {
            let removed = state.lock().unwrap().finalize_toast_removal(id);
            if removed {
                sync_toast_stack(&ui, &state);
            }
        }
    });
}

fn enqueue_toast(ui: &MainWindow, state: &Arc<Mutex<ToastQueue>>, text: String, level: i32) {
    let notice = ToastNotice {
        text,
        level,
        closing: false,
    };
    let (id, evicted_id) = state.lock().unwrap().push_toast(notice);

    sync_toast_stack(ui, state);

    schedule_auto_dismiss(ui.as_weak(), state.clone(), id);

    if let Some(evicted_id) = evicted_id {
        schedule_removal(ui, state.clone(), evicted_id);
    }
}

fn sync_toast_stack(ui: &MainWindow, state: &Arc<Mutex<ToastQueue>>) {
    let wanted: Vec<ToastItem> = {
        let queue = state.lock().unwrap();
        queue
            .stack
            .iter()
            .rev()
            .map(|toast| ToastItem {
                text: toast.text.clone().into(),
                level: toast.level,
                id: toast.id as i32,
                closing: toast.closing,
            })
            .collect()
    };

    let current = ui.global::<ToastState>().get_items();
    let Some(model) = current.as_any().downcast_ref::<VecModel<ToastItem>>() else {
        ui.global::<ToastState>()
            .set_items(ModelRc::new(VecModel::from(wanted)));
        return;
    };

    let mut row = 0;
    while row < model.row_count() {
        let row_id = model.row_data(row).map(|item| item.id).unwrap_or_default();
        if wanted.iter().any(|item| item.id == row_id) {
            row += 1;
        } else {
            model.remove(row);
        }
    }

    for (row, wanted_item) in wanted.iter().enumerate() {
        match model.row_data(row) {
            Some(existing) if existing.id == wanted_item.id => {
                if existing.closing != wanted_item.closing
                    || existing.text != wanted_item.text
                    || existing.level != wanted_item.level
                {
                    model.set_row_data(row, wanted_item.clone());
                }
            }
            _ => model.insert(row, wanted_item.clone()),
        }
    }
}

fn dismiss_toast(ui: &MainWindow, state: &Arc<Mutex<ToastQueue>>, toast_id: i32) {
    let id = toast_id as u32;
    let started = state.lock().unwrap().begin_toast_closing(id);
    if !started {
        return;
    }

    sync_toast_stack(ui, state);

    schedule_removal(ui, state.clone(), id);
}

fn schedule_auto_dismiss(ui: slint::Weak<MainWindow>, state: Arc<Mutex<ToastQueue>>, id: u32) {
    slint::Timer::single_shot(TOAST_LIFETIME, move || {
        if let Some(ui) = ui.upgrade() {
            let still_present = state
                .lock()
                .unwrap()
                .stack
                .iter()
                .any(|toast| toast.id == id);
            if !still_present {
                return;
            }
            dismiss_toast(&ui, &state, id as i32);
        }
    });
}
