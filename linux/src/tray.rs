//! The panel item: org.kde.StatusNotifierItem + com.canonical.dbusmenu.
//! The text next to the icon (XAyatanaLabel) shows today's distance where
//! the panel supports it (GNOME AppIndicator extension, Unity, Budgie);
//! elsewhere it is in the tooltip.

use crate::dbus::{Arg, Bus, Handler, V};
use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub const ITEM_PATH: &str = "/StatusNotifierItem";
pub const MENU_PATH: &str = "/MenuBar";
const ITEM_IFACE: &str = "org.kde.StatusNotifierItem";
const MENU_IFACE: &str = "com.canonical.dbusmenu";
const WATCHER: &str = "org.kde.StatusNotifierWatcher";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Open(&'static str),
    Quit,
}

pub struct Tray {
    label: Mutex<String>,
    actions: Mutex<Sender<Action>>,
    /// Menu texts in the current language, by item id (3 is the separator).
    menu: Mutex<[String; 5]>,
    revision: Mutex<u32>,
}

const ICONS: [(i32, &[u8]); 4] = [
    (22, include_bytes!("../assets/icon-22.argb")),
    (32, include_bytes!("../assets/icon-32.argb")),
    (48, include_bytes!("../assets/icon-48.argb")),
    (64, include_bytes!("../assets/icon-64.argb")),
];

// item ids; 3 is a separator
const MENU: [i32; 4] = [1, 2, 3, 4];

impl Tray {
    pub fn new(actions: Sender<Action>) -> Tray {
        let menu = [String::new(), "Statistika".into(), "Sozlamalar".into(), String::new(), "Odomouse'dan chiqish".into()];
        Tray { label: Mutex::new(String::new()), actions: Mutex::new(actions), menu: Mutex::new(menu), revision: Mutex::new(1) }
    }

    fn act(&self, a: Action) {
        let _ = self.actions.lock().unwrap().send(a);
    }

    fn tooltip(&self) -> V {
        let label = self.label.lock().unwrap().clone();
        V::Struct(vec![V::s(""), V::Arr("(iiay)".into(), vec![]), V::s("Odomouse"), V::s(&label)])
    }

    fn pixmaps() -> V {
        V::Arr(
            "(iiay)".into(),
            ICONS
                .iter()
                .map(|(size, data)| V::Struct(vec![V::I(*size), V::I(*size), V::Arr("y".into(), data.iter().map(|b| V::Y(*b)).collect())]))
                .collect(),
        )
    }

    fn item_prop(&self, name: &str) -> Option<V> {
        Some(match name {
            "Category" => V::s("ApplicationStatus"),
            "Id" => V::s("odomouse"),
            "Title" => V::s("Odomouse"),
            "Status" => V::s("Active"),
            "WindowId" => V::I(0),
            "IconName" => V::s(""),
            "IconThemePath" => V::s(""),
            "IconPixmap" => Tray::pixmaps(),
            "OverlayIconName" | "AttentionIconName" | "AttentionMovieName" => V::s(""),
            "OverlayIconPixmap" | "AttentionIconPixmap" => V::Arr("(iiay)".into(), vec![]),
            "ToolTip" => self.tooltip(),
            "ItemIsMenu" => V::B(false),
            "Menu" => V::O(MENU_PATH.into()),
            "XAyatanaLabel" => V::S(self.label.lock().unwrap().clone()),
            "XAyatanaLabelGuide" => V::s("0000.00 km"),
            "XAyatanaOrderingIndex" => V::U(0),
            _ => return None,
        })
    }

    const ITEM_PROPS: [&'static str; 18] = [
        "Category", "Id", "Title", "Status", "WindowId", "IconName", "IconThemePath", "IconPixmap",
        "OverlayIconName", "AttentionIconName", "AttentionMovieName", "OverlayIconPixmap", "AttentionIconPixmap",
        "ToolTip", "ItemIsMenu", "Menu", "XAyatanaLabel", "XAyatanaLabelGuide",
    ];

    fn menu_prop(name: &str) -> Option<V> {
        Some(match name {
            "Version" => V::U(3),
            "TextDirection" => V::s("ltr"),
            "Status" => V::s("normal"),
            "IconThemePath" => V::Arr("s".into(), vec![]),
            _ => return None,
        })
    }

    fn menu_item_props(&self, id: i32) -> V {
        match id {
            0 => V::dict(vec![("children-display", V::s("submenu"))]),
            3 => V::dict(vec![("type", V::s("separator"))]),
            1 | 2 | 4 => {
                let label = self.menu.lock().unwrap()[id as usize].clone();
                V::dict(vec![("label", V::S(label)), ("enabled", V::B(true)), ("visible", V::B(true))])
            }
            _ => V::dict(vec![]),
        }
    }

    fn layout(&self) -> V {
        let children = MENU
            .iter()
            .map(|id| V::var(V::Struct(vec![V::I(*id), self.menu_item_props(*id), V::Arr("v".into(), vec![])])))
            .collect();
        V::Struct(vec![V::I(0), self.menu_item_props(0), V::Arr("v".into(), children)])
    }

    /// Menu texts for a new language: [stats, settings, quit].
    pub fn set_menu_texts(&self, bus: &Bus, texts: [&str; 3]) {
        {
            let mut m = self.menu.lock().unwrap();
            if m[1] == texts[0] && m[2] == texts[1] && m[4] == texts[2] {
                return;
            }
            m[1] = texts[0].to_string();
            m[2] = texts[1].to_string();
            m[4] = texts[2].to_string();
        }
        let rev = {
            let mut r = self.revision.lock().unwrap();
            *r += 1;
            *r
        };
        bus.signal(MENU_PATH, MENU_IFACE, "LayoutUpdated", &[V::U(rev), V::I(0)]);
    }

    /// Update the text next to the icon; returns true if it changed.
    pub fn set_label(&self, bus: &Bus, text: &str) -> bool {
        {
            let mut l = self.label.lock().unwrap();
            if *l == text {
                return false;
            }
            *l = text.to_string();
        }
        bus.signal(ITEM_PATH, ITEM_IFACE, "XAyatanaNewLabel", &[V::s(text), V::s("0000.00 km")]);
        bus.signal(ITEM_PATH, ITEM_IFACE, "NewToolTip", &[]);
        true
    }

    /// Tell the panel we exist (again, if it restarted).
    pub fn register_with_watcher(bus: &Bus, service: &str) {
        bus.call_no_reply(WATCHER, "/StatusNotifierWatcher", WATCHER, "RegisterStatusNotifierItem", &[V::s(service)]);
    }

    pub fn watcher_present(bus: &Bus) -> bool {
        bus.has_owner(WATCHER)
    }
}

const INTROSPECT: &str = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
 <interface name="org.freedesktop.DBus.Introspectable"><method name="Introspect"><arg type="s" direction="out"/></method></interface>
 <interface name="org.freedesktop.DBus.Properties">
  <method name="Get"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="out"/></method>
  <method name="GetAll"><arg type="s" direction="in"/><arg type="a{sv}" direction="out"/></method>
 </interface>
 <interface name="org.kde.StatusNotifierItem">
  <property name="Category" type="s" access="read"/><property name="Id" type="s" access="read"/>
  <property name="Title" type="s" access="read"/><property name="Status" type="s" access="read"/>
  <property name="WindowId" type="i" access="read"/><property name="IconName" type="s" access="read"/>
  <property name="IconPixmap" type="a(iiay)" access="read"/><property name="ToolTip" type="(sa(iiay)ss)" access="read"/>
  <property name="ItemIsMenu" type="b" access="read"/><property name="Menu" type="o" access="read"/>
  <property name="XAyatanaLabel" type="s" access="read"/>
  <method name="Activate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
  <method name="SecondaryActivate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
  <method name="ContextMenu"><arg type="i" direction="in"/><arg type="i" direction="in"/></method>
  <method name="Scroll"><arg type="i" direction="in"/><arg type="s" direction="in"/></method>
  <signal name="NewToolTip"/><signal name="XAyatanaNewLabel"><arg type="s"/><arg type="s"/></signal>
 </interface>
 <interface name="com.canonical.dbusmenu">
  <property name="Version" type="u" access="read"/><property name="Status" type="s" access="read"/>
  <method name="GetLayout"><arg type="i" direction="in"/><arg type="i" direction="in"/><arg type="as" direction="in"/><arg type="u" direction="out"/><arg type="(ia{sv}av)" direction="out"/></method>
  <method name="GetGroupProperties"><arg type="ai" direction="in"/><arg type="as" direction="in"/><arg type="a(ia{sv})" direction="out"/></method>
  <method name="GetProperty"><arg type="i" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="out"/></method>
  <method name="Event"><arg type="i" direction="in"/><arg type="s" direction="in"/><arg type="v" direction="in"/><arg type="u" direction="in"/></method>
  <method name="AboutToShow"><arg type="i" direction="in"/><arg type="b" direction="out"/></method>
  <signal name="LayoutUpdated"><arg type="u"/><arg type="i"/></signal>
 </interface>
</node>"#;

impl Handler for Tray {
    fn call(&self, path: &str, iface: &str, member: &str, args: &[Arg]) -> Option<Vec<V>> {
        let s_arg = |i: usize| match args.get(i) {
            Some(Arg::S(s)) => s.as_str(),
            _ => "",
        };
        let i_arg = |i: usize| match args.get(i) {
            Some(Arg::I(n)) => *n,
            _ => 0,
        };
        match (iface, member) {
            ("org.freedesktop.DBus.Introspectable", "Introspect") => Some(vec![V::s(INTROSPECT)]),
            ("org.freedesktop.DBus.Properties", "Get") => {
                let v = if path == MENU_PATH { Tray::menu_prop(s_arg(1)) } else { self.item_prop(s_arg(1)) };
                v.map(|v| vec![V::var(v)])
            }
            ("org.freedesktop.DBus.Properties", "GetAll") => {
                if path == MENU_PATH {
                    Some(vec![V::dict(["Version", "TextDirection", "Status", "IconThemePath"].iter().filter_map(|n| Tray::menu_prop(n).map(|v| (*n, v))).collect())])
                } else {
                    Some(vec![V::dict(Tray::ITEM_PROPS.iter().filter_map(|n| self.item_prop(n).map(|v| (*n, v))).collect())])
                }
            }
            (ITEM_IFACE, "Activate") | (ITEM_IFACE, "SecondaryActivate") | (ITEM_IFACE, "ContextMenu") => {
                self.act(Action::Open("stats"));
                Some(vec![])
            }
            (ITEM_IFACE, "Scroll") => Some(vec![]),
            (MENU_IFACE, "GetLayout") => Some(vec![V::U(*self.revision.lock().unwrap()), self.layout()]),
            (MENU_IFACE, "GetGroupProperties") => Some(vec![V::Arr(
                "(ia{sv})".into(),
                MENU.iter().map(|id| V::Struct(vec![V::I(*id), self.menu_item_props(*id)])).collect(),
            )]),
            (MENU_IFACE, "GetProperty") => {
                Some(vec![V::var(match self.menu_item_props(i_arg(0)) {
                    V::Arr(_, entries) => entries
                        .into_iter()
                        .find_map(|e| match e {
                            V::Entry(k, v) if matches!(&*k, V::S(n) if n == s_arg(1)) => match *v {
                                V::Var(inner) => Some(*inner),
                                other => Some(other),
                            },
                            _ => None,
                        })
                        .unwrap_or(V::s("")),
                    other => other,
                })])
            }
            (MENU_IFACE, "Event") => {
                if s_arg(1) == "clicked" {
                    match i_arg(0) {
                        1 => self.act(Action::Open("stats")),
                        2 => self.act(Action::Open("settings")),
                        4 => self.act(Action::Quit),
                        _ => {}
                    }
                }
                Some(vec![])
            }
            (MENU_IFACE, "EventGroup") => Some(vec![V::Arr("i".into(), vec![])]),
            (MENU_IFACE, "AboutToShow") => Some(vec![V::B(false)]),
            (MENU_IFACE, "AboutToShowGroup") => Some(vec![V::Arr("i".into(), vec![]), V::Arr("i".into(), vec![])]),
            _ => None,
        }
    }
}
