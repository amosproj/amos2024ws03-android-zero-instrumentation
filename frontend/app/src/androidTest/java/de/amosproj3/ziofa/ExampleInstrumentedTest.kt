// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class SmokeTestUI {

    @get:Rule val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun configurationSmokeTest() {
        waitForAndClick("configuration")
        waitForAndClick("per process")
        clickAnyProcess()
    }

    private fun waitForAndClick(text: String) {
        composeTestRule.onNodeWithText(text, ignoreCase = true).assertExists()
        composeTestRule.onNodeWithText(text, ignoreCase = true).performClick()
    }

    @OptIn(ExperimentalTestApi::class)
    private fun clickAnyProcess() {
        composeTestRule.waitUntilAtLeastOneExists(hasClickAction(), 5000)
        composeTestRule.onAllNodes(hasClickAction()).onLast().performClick()
    }
}
