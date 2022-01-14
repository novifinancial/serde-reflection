// Copyright (c) Facebook, Inc. and its affiliates.

import XCTest

import SerdeTests

var tests = [XCTestCaseEntry]()
tests += SerdeTests.__allTests()

XCTMain(tests)
