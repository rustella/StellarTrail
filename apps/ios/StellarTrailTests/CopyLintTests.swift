import XCTest
@testable import StellarTrail

final class CopyLintTests: XCTestCase {
    func testForbiddenTermsAreDetected() {
        XCTAssertEqual(TrailCopyLint.violations(in: "连接地址"), [])
        XCTAssertEqual(TrailCopyLint.violations(in: "出行装备参考"), [])
        XCTAssertEqual(TrailCopyLint.violations(in: "API 后端 接口 模板 游客"), ["模板", "API", "后端", "接口", "游客"])
    }

    func testKnownVisibleCopyAvoidsForbiddenTerms() {
        let visibleCopy = TrailVisibleCopyCatalog.all
        let violations = visibleCopy.flatMap { text in
            TrailCopyLint.violations(in: text).map { "\($0) in \(text)" }
        }
        XCTAssertTrue(violations.isEmpty, violations.joined(separator: "\n"))
    }
}
