import XCTest

final class ScreenshotFlowUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testReviewScreenshotFlow() throws {
        var app = launchScreenshotApp(theme: "light")

        XCTAssertTrue(app.tabBars.buttons["首页"].waitForExistence(timeout: 8))
        XCTAssertTrue(app.staticTexts["今天准备好出发了吗？"].waitForExistence(timeout: 8))
        capture("01-home-guest-light", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["出行装备参考"].waitForExistence(timeout: 8))
        capture("02-gears-guest-light", app: app)

        tap("账号登录", app: app)
        capture("03-login-password-light", app: app)
        tap("邮箱验证码", app: app)
        capture("04-login-email-light", app: app)

        app.terminate()
        app = launchScreenshotApp(signedIn: true, theme: "light")

        captureSignedInParityFlow(app, suffix: "light", includeSkillDetail: true)
    }

    func testDarkScreenshotFlow() throws {
        let app = launchScreenshotApp(theme: "dark")

        XCTAssertTrue(app.tabBars.buttons["首页"].waitForExistence(timeout: 8))
        XCTAssertTrue(app.staticTexts["今天准备好出发了吗？"].waitForExistence(timeout: 8))

        tapTab("首页", app: app)
        XCTAssertTrue(app.staticTexts["今天准备好出发了吗？"].waitForExistence(timeout: 8))
        capture("01-home-guest-dark", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["出行装备参考"].waitForExistence(timeout: 8))
        capture("02-gears-guest-dark", app: app)

        tapTab("行程", app: app)
        XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", "登录后可以加入多人行程")).firstMatch.waitForExistence(timeout: 8))
        capture("03-trips-guest-dark", app: app)

        tapTab("技能", app: app)
        XCTAssertTrue(app.staticTexts["技能分类"].waitForExistence(timeout: 8))
        capture("04-skills-guest-dark", app: app)

        tapTab("我的", app: app)
        capture("05-profile-guest-dark", app: app)
    }

    func testDarkSignedInScreenshotFlow() throws {
        let app = launchScreenshotApp(signedIn: true, theme: "dark")

        captureSignedInParityFlow(app, suffix: "dark", includeSkillDetail: false)
    }

    private func launchScreenshotApp(signedIn: Bool = false, theme: String) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments += ["--stellartrail-screenshot-fixtures", "--stellartrail-screenshot-\(theme)"]
        if signedIn {
            app.launchArguments += ["--stellartrail-screenshot-signed-in"]
        }
        app.launchEnvironment["STELLARTRAIL_SCREENSHOT_MODE"] = "1"
        app.launch()
        return app
    }

    private func captureSignedInParityFlow(_ app: XCUIApplication, suffix: String, includeSkillDetail: Bool) {
        tapTab("首页", app: app)
        XCTAssertTrue(app.staticTexts["我的装备已保存"].waitForExistence(timeout: 8))
        capture("05-home-signed-in-\(suffix)", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["可用装备"].waitForExistence(timeout: 8))
        capture("06-gears-available-\(suffix)", app: app)
        tapVisibleOrScroll("打包清单", app: app)
        XCTAssertTrue(app.staticTexts["武功山两日穿越"].waitForExistence(timeout: 8))
        capture("07-packing-list-\(suffix)", app: app)
        tapIdentifier("packing-row-packing-1", app: app)
        XCTAssertTrue(app.staticTexts["挑选装备"].waitForExistence(timeout: 8))
        capture("08-packing-detail-\(suffix)", app: app)
        goBack(app)
        goBack(app)

        tapTab("行程", app: app)
        XCTAssertTrue(app.staticTexts["武功山两日穿越"].waitForExistence(timeout: 8))
        capture("09-trips-\(suffix)", app: app)
        tapIdentifier("trip-row-trip-team-1", app: app)
        XCTAssertTrue(app.staticTexts["公共装备"].waitForExistence(timeout: 8))
        capture("10-trip-detail-team-\(suffix)", app: app)
        tapVisibleOrScroll("公共装备", app: app)
        capture("11-trip-shared-gear-\(suffix)", app: app)
        goBack(app)

        tapTab("技能", app: app)
        XCTAssertTrue(app.staticTexts["安全说明已确认"].waitForExistence(timeout: 8))
        capture("12-skills-\(suffix)", app: app)
        if includeSkillDetail {
            tap("缓存全部", app: app)
            XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", "已缓存")).firstMatch.waitForExistence(timeout: 8))
            capture("13-skills-cache-\(suffix)", app: app)
            tapVisibleOrScroll("单套结", app: app)
            XCTAssertTrue(app.staticTexts["媒体展示"].waitForExistence(timeout: 8))
            capture("14-knot-detail-\(suffix)", app: app)
            tap("关闭", app: app)
        }

        tapTab("我的", app: app)
        capture("15-profile-signed-in-\(suffix)", app: app)
        tapVisibleOrScroll("产品路线图", app: app)
        XCTAssertTrue(app.staticTexts["client_key=ios"].waitForExistence(timeout: 8))
        capture("16-roadmap-\(suffix)", app: app)
        goBack(app)
        tapVisibleOrScroll("户外资料", app: app)
        XCTAssertTrue(app.staticTexts["紧急与健康"].waitForExistence(timeout: 8))
        capture("17-outdoor-profile-\(suffix)", app: app)
        goBack(app)
        tapVisibleOrScroll("户外经历", app: app)
        XCTAssertTrue(app.staticTexts["历史行程和复盘"].waitForExistence(timeout: 8))
        capture("18-outdoor-experiences-\(suffix)", app: app)
        goBack(app)
    }

    private func tapTab(_ title: String, app: XCUIApplication) {
        let tabBar = app.tabBars.firstMatch
        XCTAssertTrue(tabBar.waitForExistence(timeout: 6), "Missing tab bar")
        let tabButton = app.tabBars.buttons[title].firstMatch
        if tabButton.waitForExistence(timeout: 6) {
            let deadline = Date().addingTimeInterval(3)
            while !tabButton.isHittable && Date() < deadline {
                RunLoop.current.run(until: Date().addingTimeInterval(0.2))
            }
            if tabButton.isHittable {
                tabButton.tap()
                return
            }
        }
        if tapFirstHittable(app.tabBars.buttons.matching(NSPredicate(format: "label == %@", title))) {
            return
        }
        let indexByTitle = ["首页": 0, "装备": 1, "行程": 2, "技能": 3, "我的": 4]
        if let index = indexByTitle[title] {
            let indexedButton = app.tabBars.buttons.element(boundBy: index)
            if indexedButton.exists {
                indexedButton.tap()
                return
            }
        }
        let offsetX: CGFloat
        switch title {
        case "首页": offsetX = 0.125
        case "装备": offsetX = 0.300
        case "行程": offsetX = 0.500
        case "技能": offsetX = 0.700
        case "我的": offsetX = 0.900
        default:
            XCTFail("Unknown tab \(title)")
            return
        }
        tabBar.coordinate(withNormalizedOffset: CGVector(dx: offsetX, dy: 0.5)).tap()
    }

    private func tap(_ label: String, app: XCUIApplication, timeout: TimeInterval = 6) {
        let deadline = Date().addingTimeInterval(timeout)
        repeat {
            if tapFirstHittable(app.buttons.matching(NSPredicate(format: "label == %@", label))) { return }
            if tapFirstHittable(app.buttons.matching(NSPredicate(format: "label CONTAINS %@", label))) { return }
            if tapFirstVisibleArea(app.buttons.matching(NSPredicate(format: "label == %@", label)), app: app) { return }
            if tapFirstVisibleArea(app.buttons.matching(NSPredicate(format: "label CONTAINS %@", label)), app: app) { return }
            if tapFirstHittable(app.staticTexts.matching(NSPredicate(format: "label == %@", label))) { return }
            RunLoop.current.run(until: Date().addingTimeInterval(0.2))
        } while Date() < deadline
        if tapFirstExistingCenter(app.staticTexts.matching(NSPredicate(format: "label == %@", label)), app: app) { return }
        XCTFail("Missing tappable element \(label)")
    }

    private func tapVisibleOrScroll(_ label: String, app: XCUIApplication, maxSwipes: Int = 8) {
        for _ in 0..<maxSwipes {
            if tapFirstHittable(app.buttons.matching(NSPredicate(format: "label == %@", label))) { return }
            if tapFirstHittable(app.buttons.matching(NSPredicate(format: "label CONTAINS %@", label))) { return }
            if tapFirstVisibleArea(app.buttons.matching(NSPredicate(format: "label == %@", label)), app: app) { return }
            if tapFirstVisibleArea(app.buttons.matching(NSPredicate(format: "label CONTAINS %@", label)), app: app) { return }
            if tapFirstHittable(app.staticTexts.matching(NSPredicate(format: "label == %@", label))) { return }
            if tapFirstExistingCenter(app.staticTexts.matching(NSPredicate(format: "label == %@", label)), app: app) { return }
            app.swipeUp()
        }
        XCTFail("Missing visible element after scrolling: \(label)")
    }

    private func tapIdentifier(_ identifier: String, app: XCUIApplication, maxSwipes: Int = 8) {
        for _ in 0..<maxSwipes {
            let query = app.descendants(matching: .any).matching(identifier: identifier)
            let element = query.firstMatch
            if element.exists {
                let deadline = Date().addingTimeInterval(2)
                while !element.isHittable && Date() < deadline {
                    RunLoop.current.run(until: Date().addingTimeInterval(0.2))
                }
                if element.isHittable {
                    element.tap()
                    return
                }
                if tapFirstVisibleArea(query, app: app) {
                    return
                }
            }
            app.swipeUp()
        }
        XCTFail("Missing identifiable element after scrolling: \(identifier)")
    }

    private func tapFirstHittable(_ query: XCUIElementQuery) -> Bool {
        for index in 0..<query.count {
            let element = query.element(boundBy: index)
            if element.exists && element.isHittable {
                element.tap()
                return true
            }
        }
        return false
    }

    private func tapFirstVisibleArea(_ query: XCUIElementQuery, app: XCUIApplication) -> Bool {
        let visibleFrame = app.windows.firstMatch.exists ? app.windows.firstMatch.frame : app.frame
        for index in 0..<query.count {
            let element = query.element(boundBy: index)
            guard element.exists, !element.frame.isEmpty else { continue }
            let visibleArea = element.frame.intersection(visibleFrame)
            guard visibleArea.width > 8, visibleArea.height > 8 else { continue }
            let point = CGPoint(x: visibleArea.midX, y: visibleArea.midY)
            let dx = (point.x - element.frame.minX) / element.frame.width
            let dy = (point.y - element.frame.minY) / element.frame.height
            element.coordinate(withNormalizedOffset: CGVector(dx: dx, dy: dy)).tap()
            return true
        }
        return false
    }

    private func tapFirstExistingCenter(_ query: XCUIElementQuery, app: XCUIApplication) -> Bool {
        let visibleFrame = app.windows.firstMatch.exists ? app.windows.firstMatch.frame : app.frame
        for index in 0..<query.count {
            let element = query.element(boundBy: index)
            let center = CGPoint(x: element.frame.midX, y: element.frame.midY)
            if element.exists && !element.frame.isEmpty && visibleFrame.contains(center) {
                element.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)).tap()
                return true
            }
        }
        return false
    }

    private func goBack(_ app: XCUIApplication) {
        let backButton = app.navigationBars.buttons.element(boundBy: 0)
        XCTAssertTrue(backButton.waitForExistence(timeout: 6), "Missing navigation back button")
        backButton.tap()
    }

    private func dismissSystemPrompts(_ app: XCUIApplication) {
        let notNow = app.buttons["Not Now"].firstMatch
        if notNow.waitForExistence(timeout: 3) {
            notNow.tap()
            _ = notNow.waitForNonExistence(timeout: 3)
            RunLoop.current.run(until: Date().addingTimeInterval(0.5))
        }
    }

    private func capture(_ name: String, app: XCUIApplication) {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
