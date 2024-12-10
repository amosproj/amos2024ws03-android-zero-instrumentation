// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

import org.jetbrains.kotlin.gradle.dsl.KotlinVersion
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    `kotlin-dsl`
    alias(libs.plugins.com.ncorti.ktfmt.gradle)
}

gradlePlugin {
    plugins {
        register("rust-uniffi") {
            id = "rust.uniffi"
            implementationClass = "de.amosproj3.ziofa.gradle.rust.CargoPlugin"
        }
    }
}

dependencies {
    compileOnly(libs.android.gradlePlugin)
}

tasks.withType<KotlinCompile>().configureEach {
    compilerOptions {
        languageVersion = KotlinVersion.KOTLIN_2_0
        apiVersion = KotlinVersion.KOTLIN_2_0
    }
}