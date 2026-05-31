import XCTest
@testable import StellarTrail

final class AppShellTests: XCTestCase {
    func testRootTabsKeepExpectedOrder() {
        XCTAssertEqual(RootTab.allCases.map(\.title), ["首页", "装备", "行程", "技能", "我的"])
    }
}
