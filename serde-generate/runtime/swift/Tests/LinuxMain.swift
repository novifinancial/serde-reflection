import XCTest

import SerdeTests

var tests = [XCTestCaseEntry]()
tests += SerdeTests.__allTests()

XCTMain(tests)
