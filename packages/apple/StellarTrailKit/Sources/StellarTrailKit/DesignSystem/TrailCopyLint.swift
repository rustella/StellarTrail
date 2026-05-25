import Foundation

enum TrailCopyLint {
    static let forbiddenTerms = ["模板", "API", "后端", "接口", "游客"]

    static func violations(in text: String) -> [String] {
        forbiddenTerms.filter { text.contains($0) }
    }
}
