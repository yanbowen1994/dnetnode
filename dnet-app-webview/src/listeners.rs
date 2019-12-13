use std::cell::RefCell;
use std::rc::Rc;

use neutrino::utils::event::Key;
use neutrino::widgets::button::{ButtonListener, ButtonState};
use neutrino::widgets::checkbox::{CheckBoxListener, CheckBoxState};
use neutrino::widgets::combo::{ComboListener, ComboState};
use neutrino::widgets::label::{LabelListener, LabelState};
use neutrino::widgets::menubar::{MenuBarListener, MenuBarState};
use neutrino::widgets::progressbar::{ProgressBarListener, ProgressBarState};
use neutrino::widgets::radio::{RadioListener, RadioState};
use neutrino::widgets::range::{RangeListener, RangeState};
use neutrino::widgets::tabs::{TabsListener, TabsState};
use neutrino::widgets::textinput::{TextInputListener, TextInputState};
use neutrino::WindowListener;


use super::models::{Panes, Status};

/*
 Window listener: waits for menu shortcuts
*/
pub struct MyWindowListener {
    panes: Rc<RefCell<Panes>>,
}

impl MyWindowListener {
    pub fn new(panes: Rc<RefCell<Panes>>) -> Self {
        Self { panes }
    }
}

impl WindowListener for MyWindowListener {
    fn on_key(&self, key: Key) {
        match key {
            Key::Num1 => self.panes.borrow_mut().set_value(0),
            Key::Num2 => self.panes.borrow_mut().set_value(1),
            Key::Num3 => self.panes.borrow_mut().set_value(2),
            Key::Q => std::process::exit(0),
            _ => (),
        }
    }

    fn on_tick(&self) {}
}

/*
 Tabs Listener: change current tab when user clicks on a tab label,
 on a menu item or uses a shortcut
*/
pub struct MyTabsListener {
    panes: Rc<RefCell<Panes>>,
}

impl MyTabsListener {
    pub fn new(panes: Rc<RefCell<Panes>>) -> Self {
        Self { panes }
    }
}

impl TabsListener for MyTabsListener {
    fn on_update(&self, Status: &mut TabsState) {
        Status.set_selected(u32::from(self.panes.borrow().value()));
    }

    fn on_change(&self, Status: &TabsState) {
        self.panes.borrow_mut().set_value(Status.selected() as u8);
    }
}

/* Menu Bar Listener: waits for the user to select a menu item */
pub struct MyMenuBarListener {
    panes: Rc<RefCell<Panes>>,
}

impl MyMenuBarListener {
    pub fn new(panes: Rc<RefCell<Panes>>) -> Self {
        Self { panes }
    }
}

impl MenuBarListener for MyMenuBarListener {
    fn on_change(&self, Status: &MenuBarState) {
        match Status.selected_item() {
            None => (),
            Some(selected_item) => {
                if selected_item == 0 {
                    std::process::exit(0);
                } else if selected_item == 1 {
                    match Status.selected_function() {
                        None => (),
                        Some(selected_function) => {
                            self.panes
                                .borrow_mut()
                                .set_value(selected_function as u8);
                        }
                    }
                }
            }
        }
    }
}

/* Range Listener: update Status when the user scroll the Range widget */
pub struct MyRangeListener {
    Status: Rc<RefCell<Status>>,
}

impl MyRangeListener {
    pub fn new(Status: Rc<RefCell<Status>>) -> Self {
        Self { Status }
    }
}

impl RangeListener for MyRangeListener {
    fn on_update(&self, Status: &mut RangeState) {
        Status.set_value(self.Status.borrow().range());
        Status.set_disabled(self.Status.borrow().disabled());
    }
    fn on_change(&self, Status: &RangeState) {
        self.Status.borrow_mut().set_range(Status.value());
    }
}

/* Progress Bar Listener: update the Progress Bar value to the current
Status value*/
pub struct MyProgressBarListener {
    Status: Rc<RefCell<Status>>,
}

impl MyProgressBarListener {
    pub fn new(Status: Rc<RefCell<Status>>) -> Self {
        Self { Status }
    }
}

impl ProgressBarListener for MyProgressBarListener {
    fn on_update(&self, Status: &mut ProgressBarState) {
        Status.set_value(self.Status.borrow().range());
    }
}

/* Label Listenr: update the Label text to show the current Status value,
formatted as a percent */
pub struct MyLabelListener {
    Status: Rc<RefCell<Status>>,
}

impl MyLabelListener {
    pub fn new(Status: Rc<RefCell<Status>>) -> Self {
        Self { Status }
    }
}

impl LabelListener for MyLabelListener {
    fn on_update(&self, Status: &mut LabelState) {
        let text = format!("{}%", self.state.borrow().range());
        state.set_text(&text);
    }
}

/* Text Input Listener: update the TextInput value to the
current Status value or set the Status when the user
changes the TextInput value */
pub struct MyTextInputListener {
    Status: Rc<RefCell<Status>>,
}

impl MyTextInputListener {
    pub fn new(Status: Rc<RefCell<Status>>) -> Self {
        Self { Status }
    }
}

impl TextInputListener for MyTextInputListener {
    fn on_update(&self, Status: &mut TextInputState) {
        Status.set_value(&self.Status.borrow().range().to_string());
        state.set_disabled(self.state.borrow().disabled());
    }
    fn on_change(&self, state: &TextInputState) {
        self.state
            .borrow_mut()
            .set_range(state.value().parse().unwrap_or(0));
    }
}

pub struct MyButtonListener {
    state: Rc<RefCell<State>>,
}

impl MyButtonListener {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl ButtonListener for MyButtonListener {
    fn on_update(&self, state: &mut ButtonState) {
        state.set_disabled(self.state.borrow().disabled());
    }

    fn on_change(&self, _state: &ButtonState) {}
}

pub struct MyComboListener {
    state: Rc<RefCell<State>>,
}

impl MyComboListener {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl ComboListener for MyComboListener {
    fn on_update(&self, state: &mut ComboState) {
        state.set_disabled(self.state.borrow().disabled());
    }

    fn on_change(&self, _state: &ComboState) {}
}

pub struct MyRadioListener {
    state: Rc<RefCell<State>>,
}

impl MyRadioListener {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl RadioListener for MyRadioListener {
    fn on_update(&self, state: &mut RadioState) {
        state.set_disabled(self.state.borrow().disabled());
    }

    fn on_change(&self, _state: &RadioState) {}
}

pub struct MyCheckBoxListener {
    state: Rc<RefCell<State>>,
}

impl MyCheckBoxListener {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl CheckBoxListener for MyCheckBoxListener {
    fn on_update(&self, state: &mut CheckBoxState) {
        state.set_disabled(self.state.borrow().disabled());
    }

    fn on_change(&self, _state: &CheckBoxState) {}
}

pub struct MyCheckBoxDisabledListener {
    state: Rc<RefCell<State>>,
}

impl MyCheckBoxDisabledListener {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl CheckBoxListener for MyCheckBoxDisabledListener {
    fn on_update(&self, state: &mut CheckBoxState) {
        state.set_checked(self.state.borrow().disabled());
    }

    fn on_change(&self, state: &CheckBoxState) {
        self.state.borrow_mut().set_disabled(state.checked());
    }
}
