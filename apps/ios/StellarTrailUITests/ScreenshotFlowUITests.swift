import XCTest

final class ScreenshotFlowUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testReviewScreenshotFlow() throws {
        let app = XCUIApplication()
        app.launchArguments += ["--stellartrail-screenshot-fixtures"]
        app.launchEnvironment["STELLARTRAIL_SCREENSHOT_MODE"] = "1"
        app.launch()

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
        tap("找回密码", app: app)
        capture("05-password-reset-light", app: app)
        tap("注册账号", app: app)
        capture("06-register-light", app: app)
        tap("账号登录", app: app)
        signIn(app)

        tapTab("首页", app: app)
        XCTAssertTrue(app.staticTexts["我的装备已保存"].waitForExistence(timeout: 8))
        capture("07-home-signed-in-light", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["可用装备"].waitForExistence(timeout: 8))
        capture("08-gears-available-light", app: app)
        tapVisibleOrScroll("历史", app: app)
        capture("09-gears-history-light", app: app)
        tapVisibleOrScroll("可用", app: app)
        app.swipeDown()

        tapVisibleOrScroll("装备图鉴", app: app)
        XCTAssertTrue(app.staticTexts["搜索筛选"].waitForExistence(timeout: 8))
        capture("10-gear-atlas-list-light", app: app)
        tapVisibleOrScroll("山野 45L 轻量背包", app: app)
        XCTAssertTrue(app.staticTexts["基本信息"].waitForExistence(timeout: 8))
        capture("11-gear-atlas-detail-light", app: app)
        goBack(app)
        tapVisibleOrScroll("投稿装备", app: app)
        XCTAssertTrue(app.staticTexts["提交公开装备信息"].waitForExistence(timeout: 8))
        capture("12-gear-atlas-submit-top-light", app: app)
        app.swipeUp()
        capture("13-gear-atlas-submit-bottom-light", app: app)
        tap("关闭", app: app)
        goBack(app)

        tapIdentifier("gear-row-gear-1", app: app)
        XCTAssertTrue(app.staticTexts["图鉴投稿"].waitForExistence(timeout: 8))
        capture("14-gear-detail-light", app: app)
        tap("编辑装备", app: app)
        capture("15-gear-edit-top-light", app: app)
        app.swipeUp()
        capture("16-gear-edit-bottom-light", app: app)
        tap("关闭", app: app)
        goBack(app)
        tap("添加", app: app)
        capture("17-gear-create-top-light", app: app)
        app.swipeUp()
        capture("18-gear-create-bottom-light", app: app)
        tap("关闭", app: app)

        tapTab("技能", app: app)
        XCTAssertTrue(app.staticTexts["技能分类"].waitForExistence(timeout: 8))
        capture("19-skills-light", app: app)
        tap("缓存全部", app: app)
        XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", "已缓存")).firstMatch.waitForExistence(timeout: 8))
        capture("20-skills-cache-light", app: app)
        tapVisibleOrScroll("单套结", app: app)
        XCTAssertTrue(app.staticTexts["媒体展示"].waitForExistence(timeout: 8))
        capture("21-knot-detail-light", app: app)
        tap("关闭", app: app)

        tapTab("我的", app: app)
        capture("22-profile-signed-in-light", app: app)
        app.swipeUp()
        capture("23-profile-edit-email-light", app: app)
        tapVisibleOrScroll("深色", app: app)
        capture("24-profile-signed-in-dark", app: app)
        tapTab("首页", app: app)
        capture("25-home-signed-in-dark", app: app)
        tapTab("装备", app: app)
        capture("26-gears-signed-in-dark", app: app)
    }

    func testDarkScreenshotFlow() throws {
        let app = XCUIApplication()
        app.launchArguments += ["--stellartrail-screenshot-fixtures"]
        app.launchEnvironment["STELLARTRAIL_SCREENSHOT_MODE"] = "1"
        app.launch()

        XCTAssertTrue(app.tabBars.buttons["首页"].waitForExistence(timeout: 8))
        XCTAssertTrue(app.staticTexts["今天准备好出发了吗？"].waitForExistence(timeout: 8))
        tapTab("我的", app: app)
        XCTAssertTrue(app.staticTexts["主题"].waitForExistence(timeout: 8))
        tapVisibleOrScroll("深色", app: app)

        tapTab("首页", app: app)
        XCTAssertTrue(app.staticTexts["今天准备好出发了吗？"].waitForExistence(timeout: 8))
        capture("01-home-guest-dark", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["出行装备参考"].waitForExistence(timeout: 8))
        capture("02-gears-guest-dark", app: app)

        tap("账号登录", app: app)
        capture("03-login-password-dark", app: app)
        tap("邮箱验证码", app: app)
        capture("04-login-email-dark", app: app)
        tap("找回密码", app: app)
        capture("05-password-reset-dark", app: app)
        tap("注册账号", app: app)
        capture("06-register-dark", app: app)
        tap("账号登录", app: app)
        signIn(app)

        tapTab("首页", app: app)
        XCTAssertTrue(app.staticTexts["我的装备已保存"].waitForExistence(timeout: 8))
        capture("07-home-signed-in-dark", app: app)

        tapTab("装备", app: app)
        XCTAssertTrue(app.staticTexts["可用装备"].waitForExistence(timeout: 8))
        capture("08-gears-available-dark", app: app)
        tapVisibleOrScroll("历史", app: app)
        capture("09-gears-history-dark", app: app)
        tapVisibleOrScroll("可用", app: app)
        app.swipeDown()

        tapVisibleOrScroll("装备图鉴", app: app)
        XCTAssertTrue(app.staticTexts["搜索筛选"].waitForExistence(timeout: 8))
        capture("10-gear-atlas-list-dark", app: app)
        tapVisibleOrScroll("山野 45L 轻量背包", app: app)
        XCTAssertTrue(app.staticTexts["基本信息"].waitForExistence(timeout: 8))
        capture("11-gear-atlas-detail-dark", app: app)
        goBack(app)
        tapVisibleOrScroll("投稿装备", app: app)
        XCTAssertTrue(app.staticTexts["提交公开装备信息"].waitForExistence(timeout: 8))
        capture("12-gear-atlas-submit-top-dark", app: app)
        app.swipeUp()
        capture("13-gear-atlas-submit-bottom-dark", app: app)
        tap("关闭", app: app)
        goBack(app)

        tapIdentifier("gear-row-gear-1", app: app)
        XCTAssertTrue(app.staticTexts["图鉴投稿"].waitForExistence(timeout: 8))
        capture("14-gear-detail-dark", app: app)
        tap("编辑装备", app: app)
        capture("15-gear-edit-top-dark", app: app)
        app.swipeUp()
        capture("16-gear-edit-bottom-dark", app: app)
        tap("关闭", app: app)
        goBack(app)
        tap("添加", app: app)
        capture("17-gear-create-top-dark", app: app)
        app.swipeUp()
        capture("18-gear-create-bottom-dark", app: app)
        tap("关闭", app: app)

        tapTab("技能", app: app)
        XCTAssertTrue(app.staticTexts["技能分类"].waitForExistence(timeout: 8))
        capture("19-skills-dark", app: app)
        tap("缓存全部", app: app)
        XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", "已缓存")).firstMatch.waitForExistence(timeout: 8))
        capture("20-skills-cache-dark", app: app)
        tapVisibleOrScroll("单套结", app: app)
        XCTAssertTrue(app.staticTexts["媒体展示"].waitForExistence(timeout: 8))
        capture("21-knot-detail-dark", app: app)
        tap("关闭", app: app)

        tapTab("我的", app: app)
        capture("22-profile-signed-in-dark", app: app)
        app.swipeUp()
        capture("23-profile-edit-email-dark", app: app)
    }

    private func signIn(_ app: XCUIApplication) {
        tap("邮箱验证码", app: app)
        let email = app.textFields["邮箱"]
        if email.waitForExistence(timeout: 3) {
            email.tap()
            email.typeText("alice@example.com")
        }
        let code = app.textFields["邮箱验证码"]
        if code.waitForExistence(timeout: 3) {
            code.tap()
            code.typeText("654321")
        }
        app.buttons["邮箱登录"].firstMatch.tap()
        XCTAssertTrue(app.staticTexts["可用装备"].waitForExistence(timeout: 8))
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
        let offsetX: CGFloat
        switch title {
        case "首页": offsetX = 0.125
        case "装备": offsetX = 0.375
        case "技能": offsetX = 0.625
        case "我的": offsetX = 0.875
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
