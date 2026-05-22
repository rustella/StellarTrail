package com.rustella.stellartrail.domain.gear

import com.rustella.stellartrail.domain.atlas.GearAtlasStatus

val GearCategory.label: String
    get() = when (this) {
        GearCategory.BACKPACK_SYSTEM -> "背负系统"
        GearCategory.SLEEP_SYSTEM -> "睡眠系统"
        GearCategory.KITCHEN_SYSTEM -> "餐厨系统"
        GearCategory.WALKING_SYSTEM -> "行走系统"
        GearCategory.CLOTHING_SYSTEM -> "衣物系统"
        GearCategory.LIGHTING_SYSTEM -> "照明系统"
        GearCategory.FIRST_AID_SYSTEM -> "急救系统"
        GearCategory.ELECTRONICS_SYSTEM -> "电子系统"
        GearCategory.TECHNICAL_GEAR -> "技术装备"
        GearCategory.OTHER_GEAR -> "其它装备"
        GearCategory.CONSUMABLE -> "消耗品"
    }

val GearStatus.label: String
    get() = when (this) {
        GearStatus.AVAILABLE -> "可用"
        GearStatus.IN_USE -> "使用中"
        GearStatus.MAINTENANCE -> "保养中"
        GearStatus.DAMAGED -> "损坏"
        GearStatus.LOST -> "遗失"
        GearStatus.RETIRED -> "退役"
        GearStatus.SOLD -> "已售出"
        GearStatus.IDLE -> "闲置"
    }

val GearShareStatus.label: String
    get() = when (this) {
        GearShareStatus.NOT_SHARED -> "未共享"
        GearShareStatus.PENDING -> "待审核"
        GearShareStatus.APPROVED -> "已通过"
        GearShareStatus.REJECTED -> "已拒绝"
        GearShareStatus.WITHDRAWN -> "已撤回"
    }

val GearAtlasStatus.label: String
    get() = when (this) {
        GearAtlasStatus.PENDING -> "审核中"
        GearAtlasStatus.APPROVED -> "已收录"
        GearAtlasStatus.REJECTED -> "已拒绝"
    }

val GearSort.label: String
    get() = when (this) {
        GearSort.CREATED_AT_DESC -> "添加时间由新到旧"
        GearSort.CREATED_AT_ASC -> "添加时间由旧到新"
        GearSort.PURCHASE_DATE_DESC -> "购买日期由新到旧"
        GearSort.NAME_ASC -> "装备名称 A-Z"
        GearSort.WEIGHT_DESC -> "重量由高到低"
        GearSort.PRICE_DESC -> "价格由高到低"
    }

val GearTab.label: String
    get() = when (this) {
        GearTab.AVAILABLE -> "可用装备"
        GearTab.HISTORY -> "历史装备"
    }

val allGearCategories: List<GearCategory> = GearCategory.entries
val allGearStatuses: List<GearStatus> = GearStatus.entries
val allGearSorts: List<GearSort> = GearSort.entries
