package com.rustella.stellartrail.domain.gear

import com.rustella.stellartrail.domain.atlas.CreateGearAtlasSubmissionRequest
import java.math.BigDecimal
import java.math.RoundingMode

/** Mutable UI-facing representation of the gear form. */
data class GearFormState(
    val category: GearCategory = GearCategory.BACKPACK_SYSTEM,
    val name: String = "",
    val brand: String = "",
    val model: String = "",
    val color: String = "",
    val material: String = "",
    val capacity: String = "",
    val size: String = "",
    val description: String = "",
    val weightG: String = "",
    val officialPrice: String = "",
    val warmthIndex: String = "",
    val waterproofIndex: String = "",
    val purchaseDate: String = "",
    val purchasePrice: String = "",
    val expiryOrWarrantyDate: String = "",
    val purchaseLocation: String = "",
    val status: GearStatus = GearStatus.AVAILABLE,
    val storageLocation: String = "",
    val tags: String = "",
    val shareEnabled: Boolean = false,
    val notes: String = "",
)

fun GearItem.toFormState(): GearFormState = GearFormState(
    category = category,
    name = name,
    brand = brand.orEmpty(),
    model = model.orEmpty(),
    color = color.orEmpty(),
    material = material.orEmpty(),
    capacity = capacity.orEmpty(),
    size = size.orEmpty(),
    description = description.orEmpty(),
    weightG = weightG?.toString().orEmpty(),
    officialPrice = officialPriceCents?.let { cents -> cents.toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString() }.orEmpty(),
    warmthIndex = warmthIndex.orEmpty(),
    waterproofIndex = waterproofIndex.orEmpty(),
    purchaseDate = purchaseDate.orEmpty(),
    purchasePrice = purchasePriceCents?.let { cents -> cents.toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString() }.orEmpty(),
    expiryOrWarrantyDate = expiryOrWarrantyDate.orEmpty(),
    purchaseLocation = purchaseLocation.orEmpty(),
    status = status,
    storageLocation = storageLocation.orEmpty(),
    tags = tags.joinToString("，"),
    shareEnabled = shareEnabled,
    notes = notes.orEmpty(),
)

fun GearFormState.toCreateRequest(): CreateGearRequest {
    val trimmedName = name.trim()
    require(trimmedName.isNotEmpty()) { "请填写装备名称" }
    return CreateGearRequest(
        category = category,
        name = trimmedName,
        brand = blankToNull(brand),
        model = blankToNull(model),
        color = blankToNull(color),
        material = blankToNull(material),
        capacity = blankToNull(capacity),
        size = blankToNull(size),
        description = blankToNull(description),
        weightG = parseOptionalInt(weightG, "重量"),
        officialPriceCents = parseOptionalPriceCents(officialPrice, "官方价格"),
        officialPriceCurrency = currencyForPrice(officialPrice),
        warmthIndex = blankToNull(warmthIndex),
        waterproofIndex = blankToNull(waterproofIndex),
        purchaseDate = blankToNull(purchaseDate),
        purchasePriceCents = parseOptionalPriceCents(purchasePrice, "购买价格"),
        purchasePriceCurrency = currencyForPrice(purchasePrice),
        expiryOrWarrantyDate = blankToNull(expiryOrWarrantyDate),
        purchaseLocation = blankToNull(purchaseLocation),
        status = status,
        storageLocation = blankToNull(storageLocation),
        specs = buildSpecs(),
        tags = parseTags(tags),
        shareEnabled = shareEnabled,
        notes = blankToNull(notes),
    )
}

fun GearFormState.toUpdateRequest(): UpdateGearRequest = toCreateRequest().let { request ->
    UpdateGearRequest(
        category = request.category,
        name = request.name,
        brand = request.brand,
        model = request.model,
        color = request.color,
        material = request.material,
        capacity = request.capacity,
        size = request.size,
        description = request.description,
        weightG = request.weightG,
        officialPriceCents = request.officialPriceCents,
        officialPriceCurrency = request.officialPriceCurrency,
        warmthIndex = request.warmthIndex,
        waterproofIndex = request.waterproofIndex,
        purchaseDate = request.purchaseDate,
        purchasePriceCents = request.purchasePriceCents,
        purchasePriceCurrency = request.purchasePriceCurrency,
        expiryOrWarrantyDate = request.expiryOrWarrantyDate,
        purchaseLocation = request.purchaseLocation,
        status = request.status,
        storageLocation = request.storageLocation,
        specs = request.specs,
        tags = request.tags,
        shareEnabled = request.shareEnabled,
        notes = request.notes,
    )
}

fun GearFormState.toAtlasSubmissionRequest(): CreateGearAtlasSubmissionRequest {
    val trimmedName = name.trim()
    require(trimmedName.isNotEmpty()) { "请填写装备名称" }
    return CreateGearAtlasSubmissionRequest(
        category = category,
        name = trimmedName,
        brand = blankToNull(brand),
        model = blankToNull(model),
        description = blankToNull(description),
        weightG = parseOptionalInt(weightG, "重量"),
        officialPriceCents = parseOptionalPriceCents(officialPrice, "官方价格"),
        officialPriceCurrency = currencyForPrice(officialPrice),
        specs = buildSpecs(),
    )
}

fun parseTags(input: String): List<String> = input
    .split(',', '，', ' ')
    .map { it.trim() }
    .filter { it.isNotEmpty() }
    .distinct()

fun formatWeight(value: Int?): String = value?.let { "${it}g" } ?: "未记录"

fun formatPrice(cents: Long?, currency: String? = "CNY"): String = cents?.let {
    val amount = it.toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString()
    when (currency?.uppercase()) {
        "USD" -> "$$amount"
        "EUR" -> "€$amount"
        "JPY" -> "¥$amount"
        "HKD" -> "HK$$amount"
        else -> "¥$amount"
    }
} ?: "未记录"

fun joinBrandModel(brand: String?, model: String?): String = listOfNotNull(
    brand?.takeIf { it.isNotBlank() },
    model?.takeIf { it.isNotBlank() },
).joinToString(" · ").ifBlank { "未记录品牌型号" }

private fun blankToNull(value: String): String? = value.trim().takeIf { it.isNotEmpty() }

private fun parseOptionalInt(value: String, label: String): Int? {
    val trimmed = value.trim()
    if (trimmed.isEmpty()) return null
    val parsed = trimmed.toIntOrNull()
    require(parsed != null && parsed >= 0) { "$label 需为非负整数" }
    return parsed
}

private fun GearFormState.buildSpecs(): Map<String, String>? {
    val specs = linkedMapOf<String, String>()
    blankToNull(capacity)?.let { specs["capacity"] = it }
    blankToNull(size)?.let { specs["size"] = it }
    blankToNull(color)?.let { specs["color"] = it }
    blankToNull(material)?.let { specs["material"] = it }
    blankToNull(warmthIndex)?.let { specs["warmth_index"] = it }
    blankToNull(waterproofIndex)?.let { specs["waterproof_index"] = it }
    return specs.takeIf { it.isNotEmpty() }
}

private fun parseOptionalPriceCents(value: String, label: String): Long? {
    val trimmed = value.trim()
    if (trimmed.isEmpty()) return null
    val amount = runCatching { BigDecimal(trimmed) }.getOrNull()
    require(amount != null && amount >= BigDecimal.ZERO) { "$label 需为非负数字" }
    return amount.movePointRight(2).setScale(0, RoundingMode.HALF_UP).longValueExact()
}

private fun currencyForPrice(value: String): String? = value.trim().takeIf { it.isNotEmpty() }?.let { "CNY" }
