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