// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package uniffi.client

import kotlinx.coroutines.runBlocking
import org.junit.Test

class Test {
    @Test fun works(): Unit = runBlocking { Client.connect("https://google.com") }
}
