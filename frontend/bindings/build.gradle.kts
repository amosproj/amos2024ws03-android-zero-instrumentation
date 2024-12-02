// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

import de.amosproj3.ziofa.gradle.rust.Profile
import de.amosproj3.ziofa.gradle.rust.Target

plugins {
   alias(libs.plugins.rust.uniffi)
}

android {
    namespace = "de.amosproj3.ziofa.binding"
    compileSdk = 35
    buildToolsVersion = "35.0.0"
    ndkVersion = "28.0.12433566"

    defaultConfig {
        minSdk = 33
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
}


dependencies {
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}


cargo {
    workspaceDirectory = rootProject.file("../rust")

    val x = libraries.register("client") {
        features = listOf("uniffi")
    }

    tasks.register("abc") {
        inputs.file(x.flatMap {
            it.outputArtifacts.flatMap {
                it[de.amosproj3.ziofa.gradle.rust.Configuration(Target.Desktop.Host, Profile.Release)]!!
            }
        })
    }

    uniffi {
        uniffiCli = binaries.register("uniffi-bindgen") {
            packageName = "uniffi-bindgen"
        }
    }
}