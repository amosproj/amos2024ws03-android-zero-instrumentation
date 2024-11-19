// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.org.cyclonedx.bom)
}

android {
    namespace = "de.amosproj3.ziofa"
    compileSdk = 35
    buildToolsVersion = "35.0.0"
    ndkVersion = "28.0.12433566"

    defaultConfig {
        applicationId = "de.amosproj3.ziofa"
        minSdk = 33
        targetSdk = 35
        versionCode = 1
        versionName = rootProject.version.toString()

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    splits {
        abi {
            isEnable = true
            isUniversalApk = false
            reset()
            include("x86_64", "arm64-v8a")
        }
    }

    lint {
        ignoreWarnings = false
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.1"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }

    flavorDimensions += "environment"
    productFlavors {
        create("realBackend") {
            dimension = "environment"
        }
        create("mockedBackend") {
            dimension = "environment"
            versionNameSuffix = ".mocked"  // Optional
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.ui)
    implementation(libs.androidx.ui.graphics)
    implementation(libs.androidx.ui.tooling.preview)
    implementation(libs.androidx.material3)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.ui.test.junit4)
    debugImplementation(libs.androidx.ui.tooling)
    debugImplementation(libs.androidx.ui.test.manifest)

    implementation(project(":client"))

    compileOnly(libs.koin.core)
    implementation(libs.koin.android)
    implementation(libs.koin.compose)
    testImplementation(libs.koin.test)
    implementation(libs.jackwharton.timber)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.vico.compose)
    implementation(libs.vico.compose.m2)
    implementation(libs.vico.compose.m3)
    implementation(libs.vico.core)
}

tasks.cyclonedxBom {
    setIncludeConfigs(listOf("runtimeClasspath"))
    setSchemaVersion("1.5")
    setProjectType("application")
    setDestination(project.file("build/reports"))
    setOutputName("bom")
    setOutputFormat("json")
    setIncludeBomSerialNumber(false)
    setIncludeLicenseText(true)
    setIncludeMetadataResolution(true)
}