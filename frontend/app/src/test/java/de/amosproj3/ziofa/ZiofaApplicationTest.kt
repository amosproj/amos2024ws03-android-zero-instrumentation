// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import org.junit.Test
import org.koin.core.annotation.KoinExperimentalAPI
import org.koin.test.KoinTest

class ZiofaApplicationTest : KoinTest {
    @OptIn(KoinExperimentalAPI::class)
    @Test
    fun checkModules() {
        // ZiofaApplication().appModule.verify()
    }
}
