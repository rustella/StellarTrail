//! Public gear template domain models and default system seed data.

use serde::{Deserialize, Serialize};

use crate::locale::Locale;

/// Public gear template returned by unauthenticated read APIs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GearTemplate {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub categories: Vec<GearTemplateCategory>,
}

/// Public gear template category returned inside a template.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GearTemplateCategory {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub items: Vec<String>,
}

/// Seed payload for one DB-backed gear template.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GearTemplateSeed {
    pub id: String,
    pub title: String,
    pub localizations: Vec<(Locale, String)>,
    pub sort_order: i32,
    pub categories: Vec<GearTemplateCategorySeed>,
}

/// Seed payload for one template category.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GearTemplateCategorySeed {
    pub id: String,
    pub name: String,
    pub localizations: Vec<(Locale, String)>,
    pub sort_order: i32,
    pub items: Vec<GearTemplateItemSeed>,
}

/// Seed payload for one template item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GearTemplateItemSeed {
    pub id: String,
    pub name: String,
    pub localizations: Vec<(Locale, String)>,
    pub sort_order: i32,
}

/// Returns the built-in system gear templates seeded into the database at API startup.
pub fn default_system_gear_templates() -> Vec<GearTemplateSeed> {
    vec![GearTemplateSeed {
        id: "backpacking-basic".to_owned(),
        title: "入门徒步基础装备模板".to_owned(),
        localizations: vec![
            (Locale::ZhCn, "入门徒步基础装备模板".to_owned()),
            (
                Locale::En,
                "Beginner Backpacking Essentials Template".to_owned(),
            ),
        ],
        sort_order: 10,
        categories: vec![
            GearTemplateCategorySeed {
                id: "rain_protection".to_owned(),
                name: "防雨防风".to_owned(),
                localizations: vec![
                    (Locale::ZhCn, "防雨防风".to_owned()),
                    (Locale::En, "Rain and Wind Protection".to_owned()),
                ],
                sort_order: 10,
                items: vec![
                    GearTemplateItemSeed {
                        id: "rain-shell".to_owned(),
                        name: "雨衣或硬壳".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "雨衣或硬壳".to_owned()),
                            (Locale::En, "Rain shell or hardshell".to_owned()),
                        ],
                        sort_order: 10,
                    },
                    GearTemplateItemSeed {
                        id: "pack-cover".to_owned(),
                        name: "背包防雨罩".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "背包防雨罩".to_owned()),
                            (Locale::En, "Pack cover".to_owned()),
                        ],
                        sort_order: 20,
                    },
                ],
            },
            GearTemplateCategorySeed {
                id: "lighting".to_owned(),
                name: "照明".to_owned(),
                localizations: vec![
                    (Locale::ZhCn, "照明".to_owned()),
                    (Locale::En, "Lighting".to_owned()),
                ],
                sort_order: 20,
                items: vec![
                    GearTemplateItemSeed {
                        id: "headlamp".to_owned(),
                        name: "头灯".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "头灯".to_owned()),
                            (Locale::En, "Headlamp".to_owned()),
                        ],
                        sort_order: 10,
                    },
                    GearTemplateItemSeed {
                        id: "backup-power".to_owned(),
                        name: "备用电池或充电宝".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "备用电池或充电宝".to_owned()),
                            (Locale::En, "Spare batteries or power bank".to_owned()),
                        ],
                        sort_order: 20,
                    },
                ],
            },
            GearTemplateCategorySeed {
                id: "emergency".to_owned(),
                name: "应急".to_owned(),
                localizations: vec![
                    (Locale::ZhCn, "应急".to_owned()),
                    (Locale::En, "Emergency".to_owned()),
                ],
                sort_order: 30,
                items: vec![
                    GearTemplateItemSeed {
                        id: "first-aid-kit".to_owned(),
                        name: "急救包".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "急救包".to_owned()),
                            (Locale::En, "First aid kit".to_owned()),
                        ],
                        sort_order: 10,
                    },
                    GearTemplateItemSeed {
                        id: "emergency-blanket".to_owned(),
                        name: "保温毯".to_owned(),
                        localizations: vec![
                            (Locale::ZhCn, "保温毯".to_owned()),
                            (Locale::En, "Emergency blanket".to_owned()),
                        ],
                        sort_order: 20,
                    },
                ],
            },
        ],
    }]
}
