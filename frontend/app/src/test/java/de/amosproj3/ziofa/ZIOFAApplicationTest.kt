// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import org.junit.Test
import org.koin.test.KoinTest
import org.koin.test.verify.verify

class ZIOFAApplicationTest: KoinTest {
    @Test
    fun checkModules(){
        ZIOFAApplication().appModule.verify()
    }
}