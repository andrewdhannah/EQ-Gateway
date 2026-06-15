package com.example.eqgateway.ui.chat

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import org.junit.Before
import org.junit.Rule
import org.junit.Test

/** UI tests for [ChatScreen]. */
class ChatScreenTest {

  @get:Rule val composeTestRule = createAndroidComposeRule<ComponentActivity>()

  @Before
  fun setup() {
    composeTestRule.setContent { ChatScreen() }
  }

  @Test
  fun chatScreen_showsTitle() {
    composeTestRule.onNodeWithText("EQ Gateway Chat").assertExists()
  }

  @Test
  fun chatScreen_showsInputField() {
    composeTestRule.onNodeWithText("Type a message...").assertExists()
  }

  @Test
  fun chatScreen_showsSendButton() {
    composeTestRule.onNodeWithText("Send").assertExists()
  }
}
