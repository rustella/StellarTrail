import Foundation

enum Formatters {
    static func weight(_ grams: Int?) -> String {
        guard let grams else { return "未记录" }
        if grams >= 1_000 {
            let kg = Double(grams) / 1_000.0
            return "\(trimTrailingZeros(String(format: "%.2f", kg))) kg"
        }
        return "\(grams) g"
    }

    static func price(_ cents: Int?, currency: String? = "CNY") -> String {
        guard let cents else { return "未记录" }
        let normalized = GearCurrency.normalized(currency)
        if normalized == .jpy {
            return "\(normalized.rawValue) \(cents)"
        }
        let amount = cents % 100 == 0
            ? "\(cents / 100)"
            : String(format: "%.2f", Double(cents) / 100.0)
        if normalized == .cny {
            return "¥\(amount)"
        }
        return "\(normalized.rawValue) \(amount)"
    }

    static func weightInputText(_ grams: Int?, unit: GearWeightUnit) -> String {
        guard let grams else { return "" }
        let value: Double
        switch unit {
        case .g: value = Double(grams)
        case .kg: value = Double(grams) / 1_000
        case .lb: value = Double(grams) / 453.59237
        case .oz: value = Double(grams) / 28.349523125
        }
        return trimTrailingZeros(String(format: "%.2f", value))
    }

    static func priceInputText(_ cents: Int?, currency: String?) -> String {
        guard let cents else { return "" }
        if GearCurrency.normalized(currency) == .jpy {
            return "\(cents)"
        }
        if cents % 100 == 0 {
            return "\(cents / 100)"
        }
        return trimTrailingZeros(String(format: "%.2f", Double(cents) / 100.0))
    }

    static func brandModel(brand: String?, model: String?) -> String {
        [brand?.nilIfBlank, model?.nilIfBlank].compactMap { $0 }.joined(separator: " · ")
    }

    static func date(_ value: String?) -> String {
        value?.nilIfBlank.map { String($0.prefix(10)) } ?? "未记录"
    }

    static func dateRange(startDate: String?, endDate: String?) -> String {
        let start = startDate?.nilIfBlank.map { String($0.prefix(10)) }
        let end = endDate?.nilIfBlank.map { String($0.prefix(10)) }
        switch (start, end) {
        case let (start?, end?) where start != end:
            return "\(start) 至 \(end)"
        case let (start?, _):
            return start
        case let (_, end?):
            return end
        default:
            return "未设置时间"
        }
    }

    static func localDateString(_ date: Date = Date()) -> String {
        let formatter = DateFormatter()
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = .current
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter.string(from: date)
    }

    static func bytes(_ value: Int?) -> String {
        guard let value, value > 0 else { return "0 B" }
        let units = ["B", "KB", "MB", "GB"]
        var amount = Double(value)
        var unitIndex = 0
        while amount >= 1024, unitIndex < units.count - 1 {
            amount /= 1024
            unitIndex += 1
        }
        return "\(trimTrailingZeros(String(format: "%.1f", amount))) \(units[unitIndex])"
    }

    private static func trimTrailingZeros(_ value: String) -> String {
        value.replacingOccurrences(of: "\\.0+$", with: "", options: .regularExpression)
            .replacingOccurrences(of: "(\\.\\d*?)0+$", with: "$1", options: .regularExpression)
    }
}

extension String {
    var nilIfBlank: String? {
        let trimmed = trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }
}
