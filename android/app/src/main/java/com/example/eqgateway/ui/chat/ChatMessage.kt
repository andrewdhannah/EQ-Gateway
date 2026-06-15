package com.example.eqgateway.ui.chat

import com.example.eqgateway.model.EqState

/**
 * A single message in the chat conversation.
 */
data class ChatMessage(
    val id: Long,
    val text: String,
    val isUser: Boolean,
    val timestamp: Long = System.currentTimeMillis(),
    val eqState: EqState? = null,  // Only set for AI responses
)
