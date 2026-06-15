import httpx
import json
import logging
from typing import List, Dict, Any, Optional
from mcp.server.fastmcp import FastMCP

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("mcp-bridge")

# Configuration: Relay to the Agent Gateway service
GATEWAY_URL = "http://127.0.0.1:8090"

# Initialize FastMCP server
mcp = FastMCP("EQ-Gateway-Bridge")

async def call_gateway(endpoint: str, method: str = "POST", data: Optional[Dict] = None) -> Dict:
    """Helper to perform HTTP calls to the Agent Gateway service."""
    url = f"{GATEWAY_URL}{endpoint}"
    async with httpx.AsyncClient(timeout=30.0) as client:
        if method == "POST":
            response = await client.post(url, json=data)
        else:
            response = await client.get(url)
        
        response.raise_for_status()
        return response.json()

@mcp.tool()
async def analyze_user_input(text: str) -> str:
    """
    Analyzes raw user text to determine emotional state (affect), 
    cognitive intent, and risk level without exposing the raw text to the cloud.
    
    Returns a JSON string containing the EQ State.
    """
    logger.info(f"MCP Tool: analyze_user_input for text[:20]... {text[:20]}")
    try:
        result = await call_gateway("/analyze", data={"message": text})
        return json.dumps(result, indent=2)
    except Exception as e:
        logger.error(f"analyze_user_input error: {e}")
        return f"Error analyzing input: {str(e)}"

@mcp.tool()
async def secure_chat(text: str, session_id: str = "default-session") -> str:
    """
    Processes a user message through the EQ Gateway routed pipeline.
    The pipeline handles PII scanning, risk routing, and adaptive response generation.
    
    Returns the final routed response and the routing decision.
    """
    logger.info(f"MCP Tool: secure_chat for session {session_id}")
    try:
        result = await call_gateway("/chat", data={"message": text, "session_id": session_id})
        # We return the result as a formatted string for the LLM
        return json.dumps(result, indent=2)
    except Exception as e:
        logger.error(f"secure_chat error: {e}")
        return f"Error in secure chat: {str(e)}"

@mcp.tool()
async def list_pending_reviews() -> str:
    """
    Lists all requests currently paused for human review (HITL).
    Use this to identify messages that were flagged as 'medium' risk.
    """
    logger.info("MCP Tool: list_pending_reviews")
    try:
        result = await call_gateway("/pending", method="GET")
        return json.dumps(result, indent=2)
    except Exception as e:
        logger.error(f"list_pending_reviews error: {e}")
        return f"Error listing reviews: {str(e)}"

@mcp.tool()
async def resolve_review(request_id: str, decision: str) -> str:
    """
    Resolves a pending human review request.
    
    Args:
        request_id: The ID of the pending request.
        decision: One of 'approve', 'reject', or 'edit' (with content).
    """
    logger.info(f"MCP Tool: resolve_review for {request_id} -> {decision}")
    try:
        result = await call_gateway("/human-decision", data={"request_id": request_id, "decision": decision})
        return json.dumps(result, indent=2)
    except Exception as e:
        logger.error(f"resolve_review error: {e}")
        return f"Error resolving review: {str(e)}"

@mcp.resource("eq://status")
def get_engine_status() -> str:
    """
    Returns the current health and diagnostic status of the local EQ Engine.
    """
    # Since resource getters are typically sync in basic FastMCP, 
    # we'd normally need an async wrapper or a different approach.
    # For this POC bridge, we'll stick to tools for the main logic.
    return "Engine status resource available via analyze_user_input tool."

if __name__ == "__main__":
    # Run the server using stdio transport for MCP clients (like Claude Desktop)
    mcp.run(transport="stdio")
