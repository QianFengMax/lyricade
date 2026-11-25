use adw::prelude::*;
use relm4::{gtk::EventControllerKey, prelude::*};
use tracing::{debug, trace};

use crate::manage::{ManageLyricsOptions, ManageLyricsTarget};

pub struct ManageLyricsModel {
  state: ManageLyricsOptions,
  default_state: ManageLyricsOptions,
  alert_dialog: adw::AlertDialog,
}

#[derive(Debug)]
pub enum ManageLyricsMsg {
  UpdateState(ExposedSetting),
  ShowConfirmDialog,
  Confirm,
}

#[derive(Debug)]
pub enum ManageLyricsOutput {
  Close,
  Confirm(ManageLyricsOptions),
}

#[derive(Debug)]
pub enum ExposedSetting {
  TagsDelete(ManageLyricsTarget),
  TagsDeleteCondition(Option<ManageLyricsTarget>),
  TagsCopy(ManageLyricsTarget),
  TagsConvertToPlain(bool),
  SidecarsDelete(ManageLyricsTarget),
  SidecarsDeleteCondition(Option<ManageLyricsTarget>),
  SidecarsCopy(ManageLyricsTarget),
  SidecarsConvertToPlain(bool),
}

#[relm4::component(pub)]
impl Component for ManageLyricsModel {
  type Input = ManageLyricsMsg;
  type Output = ManageLyricsOutput;
  type CommandOutput = ();
  type Init = ();

  view! {
    adw::Window {
      set_title: Some("管理歌词"),
      set_default_width: 600,

      #[wrap(Some)]
      set_content = &adw::ToolbarView {
        add_top_bar = &adw::HeaderBar {},

        #[wrap(Some)]
        set_content = &adw::PreferencesPage {
          adw::PreferencesGroup {
            set_title: "歌词标签",
            set_description: Some("对音频文件中嵌入的歌词标签执行的操作。"),

            gtk::ListBox {
              add_css_class: "boxed-list",

              adw::ComboRow {
                set_title: "_删除歌词标签",
                set_use_underline: true,
                set_subtitle: "从音频文件删除歌词标签",
                set_model: Some(&gtk::StringList::new(&[
                  "不执行操作",
                  "删除纯文本",
                  "删除同步",
                  "删除全部",
                ])),
                #[watch]
                set_selected: model.state.tags.delete as u32,

                connect_selected_item_notify[sender] => move |row| {
                  let target = ManageLyricsTarget::from(row.selected());
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::TagsDelete(target)));
                },
              },

              adw::ComboRow {
                set_title: "_条件删除",
                set_use_underline: true,
                set_subtitle: "仅当附带文件存在时删除标签",
                set_model: Some(&gtk::StringList::new(&[
                  "Unconditional",
                  "无附带文件",
                  "纯文本附带文件",
                  "同步附带文件",
                  "任意附带文件",
                ])),
                #[watch]
                set_sensitive: model.state.tags.delete != ManageLyricsTarget::None,
                #[watch]
                set_selected: if model.state.tags.delete == ManageLyricsTarget::None { 0 }
                  else if let Some(target) = model.state.tags.delete_on_sidecar_condition { target as u32 + 1}
                  else { 0 },

                  connect_selected_item_notify[sender] => move |row| {
                  let target = match row.selected() {
                    0 => None,
                    idx => Some(ManageLyricsTarget::from(idx - 1))
                  };
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::TagsDeleteCondition(target)));
                },
              },

              adw::ComboRow {
                set_title: "_从附带文件复制",
                set_use_underline: true,
                set_subtitle: "从旁边的 LRC/TXT 文件复制到歌词标签",
                set_model: Some(&gtk::StringList::new(&[
                  "不执行操作",
                  "复制纯文本",
                  "复制同步",
                  "复制任意",
                ])),
                #[watch]
                set_sensitive: model.state.tags.delete == ManageLyricsTarget::None,
                #[watch]
                set_selected: if model.state.tags.delete == ManageLyricsTarget::None { model.state.tags.copy as u32 } else { 0 },

                connect_selected_item_notify[sender] => move |row| {
                  let target = ManageLyricsTarget::from(row.selected());
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::TagsCopy(target)));
                },
              },

              adw::SwitchRow {
                set_title: "将同步歌词_转换为纯文本",
                set_use_underline: true,
                #[watch]
                set_sensitive: model.state.tags.delete == ManageLyricsTarget::None,
                set_subtitle: "将同步 LRC 歌词标签转换为纯 TXT 歌词",
                #[watch]
                set_active: model.state.tags.convert_to_plain && model.state.tags.delete == ManageLyricsTarget::None,
                connect_active_notify[sender] => move |btn| {
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::TagsConvertToPlain(btn.is_active())));
                },
              },
            },
          },

          adw::PreferencesGroup {
            set_title: "附带文件",
            set_description: Some("对 LRC/TXT 歌词文件执行的操作。"),

            gtk::ListBox {
              add_css_class: "boxed-list",

              adw::ComboRow {
                set_title: "删除_附带文件",
                set_use_underline: true,
                set_subtitle: "删除同步 LRC 或纯文本 TXT 附带歌词文件",
                set_model: Some(&gtk::StringList::new(&[
                  "不执行操作",
                  "删除纯文本",
                  "删除同步",
                  "删除全部",
                ])),
                #[watch]
                set_selected: model.state.sidecars.delete as u32,

                connect_selected_item_notify[sender] => move |row| {
                  let target = ManageLyricsTarget::from(row.selected());
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::SidecarsDelete(target)));
                },
              },

              adw::ComboRow {
                set_title: "_条件删除",
                set_use_underline: true,
                set_subtitle: "仅当标签存在时删除附带文件",
                set_model: Some(&gtk::StringList::new(&[
                  "Unconditional",
                  "无标签",
                  "纯文本标签",
                  "同步标签",
                  "任意标签",
                ])),
                #[watch]
                set_sensitive: model.state.sidecars.delete != ManageLyricsTarget::None,
                #[watch]
                set_selected: if model.state.sidecars.delete == ManageLyricsTarget::None { 0 }
                  else if let Some(target) = model.state.sidecars.delete_on_tag_condition { target as u32 + 1}
                  else { 0 },

                  connect_selected_item_notify[sender] => move |row| {
                  let target = match row.selected() {
                    0 => None,
                    idx => Some(ManageLyricsTarget::from(idx - 1))
                  };
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::SidecarsDeleteCondition(target)));
                },
              },

              adw::ComboRow {
                set_title: "从_歌词标签复制",
                set_use_underline: true,
                set_subtitle: "从歌词标签复制到 LRC/TXT 文件",
                set_model: Some(&gtk::StringList::new(&[
                  "不执行操作",
                  "复制纯文本",
                  "复制同步",
                  "复制任意",
                ])),
                #[watch]
                set_sensitive: model.state.sidecars.delete == ManageLyricsTarget::None,
                #[watch]
                set_selected: if model.state.sidecars.delete == ManageLyricsTarget::None { model.state.sidecars.copy as u32 } else { 0 },

                connect_selected_item_notify[sender] => move |row| {
                  let target = ManageLyricsTarget::from(row.selected());
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::SidecarsCopy(target)));
                },
              },

              adw::SwitchRow {
                set_title: "将同步歌词转换为_纯文本",
                set_use_underline: true,
                set_subtitle: "将同步 LRC 歌词文件转换为纯 TXT 歌词",
                #[watch]
                set_sensitive: model.state.sidecars.delete == ManageLyricsTarget::None,
                #[watch]
                set_active: model.state.sidecars.convert_to_plain && model.state.sidecars.delete == ManageLyricsTarget::None,
                connect_active_notify[sender] => move |btn| {
                  sender.input(ManageLyricsMsg::UpdateState(ExposedSetting::SidecarsConvertToPlain(btn.is_active())));
                },
              },
            },
          },

          adw::PreferencesGroup {
            gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
              set_halign: gtk::Align::Center,
              set_spacing: 12,

              gtk::Button {
                set_label: "取消",

                connect_clicked[sender] => move |_btn| {
                  sender.output(ManageLyricsOutput::Close)
                    .expect("ManageLyricsOutput receiver dropped");
                },
              },

              gtk::Button {
                set_label: "应用",
                #[watch]
                set_sensitive: model.state != model.default_state,
                add_css_class: "suggested-action",

                connect_clicked[sender] => move |_btn| {
                  sender.input(ManageLyricsMsg::ShowConfirmDialog);
                },
              },
            },
          },
        },
      },
    },
  }

  fn init(
    _init: Self::Init,
    root: Self::Root,
    sender: ComponentSender<Self>,
  ) -> ComponentParts<Self> {
    let alert_dialog = adw::AlertDialog::builder()
      .heading("确定吗？")
      .body("此操作无法撤销。")
      .default_response("cancel")
      .close_response("cancel")
      .build();
    alert_dialog.add_response("cancel", "取消");
    alert_dialog.add_response("confirm", "Confirm");
    alert_dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);

    let sender_handle = sender.clone();
    alert_dialog.connect_response(None, move |_, resp| {
      if resp == "confirm" {
        sender_handle.input(ManageLyricsMsg::Confirm);
      }
    });

    let model = Self {
      state: ManageLyricsOptions::default(),
      default_state: ManageLyricsOptions::default(),
      alert_dialog,
    };

    let widgets = view_output!();

    // Handle key presses
    let controller = EventControllerKey::new();
    controller.connect_key_pressed(move |_con, key, _idx, modifier| {
      trace!("ViewLyrics key event: key {key} + {:?}", modifier);
      if key == gtk::gdk::Key::Escape {
        sender
          .output(ManageLyricsOutput::Close)
          .expect("ManageLyricsOutput receiver dropped");
      }
      gtk::glib::Propagation::Proceed
    });
    root.add_controller(controller);

    ComponentParts { model, widgets }
  }

  fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
    match message {
      ManageLyricsMsg::UpdateState(setting) => {
        trace!("ManageLyrics: Updating state with setting: {:?}", &setting);

        match setting {
          ExposedSetting::TagsDelete(target) => self.state.tags.delete = target,
          ExposedSetting::TagsDeleteCondition(target) => {
            self.state.tags.delete_on_sidecar_condition = target;
          }
          ExposedSetting::TagsCopy(target) => self.state.tags.copy = target,
          ExposedSetting::TagsConvertToPlain(enabled) => self.state.tags.convert_to_plain = enabled,
          ExposedSetting::SidecarsDelete(target) => self.state.sidecars.delete = target,
          ExposedSetting::SidecarsDeleteCondition(target) => {
            self.state.sidecars.delete_on_tag_condition = target;
          }
          ExposedSetting::SidecarsCopy(target) => self.state.sidecars.copy = target,
          ExposedSetting::SidecarsConvertToPlain(enabled) => {
            self.state.sidecars.convert_to_plain = enabled;
          }
        }

        debug!("ManageLyrics: Updated state:\n{:#?}", &self.state);
      }

      ManageLyricsMsg::ShowConfirmDialog => {
        debug!("ManageLyrics: Showing confirmation dialog");
        self.alert_dialog.present(Some(root));
      }

      ManageLyricsMsg::Confirm => {
        debug!("ManageLyrics: Confirmed");

        sender
          .output(ManageLyricsOutput::Confirm(self.state))
          .expect("ManageLyricsOutput receiver dropped");
      }
    }
  }
}
