import Foundation

enum Formatters {
    static func weight(_ grams: Int?) -> String {
        guard let grams else { return "未填写" }
        if grams >= 1_000 {
            let kg = Double(grams) / 1_000.0
            return String(format: "%.1f kg", kg)
        }
        return "\(grams) g"
    }

    static func price(_ cents: Int?) -> String {
        guard let cents else { return "未填写" }
        if cents % 100 == 0 {
            return "¥\(cents / 100)"
        }
        return String(format: "¥%.2f", Double(cents) / 100.0)
    }

    static func brandModel(brand: String?, model: String?) -> String {
        [brand?.nilIfBlank, model?.nilIfBlank].compactMap { $0 }.joined(separator: " · ")
    }
}

extension String {
    var nilIfBlank: String? {
        let trimmed = trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }
}
