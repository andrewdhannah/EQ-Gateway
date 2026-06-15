import asyncio
import json
import pytest
from mcp_bridge.server import analyze_user_input, secure_chat, list_pending_reviews, resolve_review

@pytest.mark.asyncio
async def test_analyze_input():
    """Verify the bridge correctly relays an analysis request."""
    text = "I am feeling very stressed about work"
    result_json = await analyze_user_input(text)
    result = json.loads(result_json)
    
    assert "eq_state" in result
    assert "affect" in result["eq_state"]
    print(f"Analysis test success: {result['eq_state']['affect']['primary']}")

@pytest.mark.asyncio
async def test_secure_chat():
    """Verify the bridge correctly relays a chat request."""
    text = "Hello there!"
    result_json = await secure_chat(text)
    result = json.loads(result_json)
    
    assert "response_text" in result
    assert "route" in result
    print(f"Chat test success: {result['route']['decision']}")

@pytest.mark.asyncio
async def test_hitl_flow():
    """Verify the bridge can list and resolve pending reviews."""
    # 1. Trigger a medium risk request to create a pending review
    # (Using a known medium-risk phrase from the Agent Gateway's router)
    await secure_chat("I need a critical review of my workplace mental health status", "test-hitl")
    
    # 2. List pending
    pending_json = await list_pending_reviews()
    pending = json.loads(pending_json)
    assert "pending" in pending
    
    if len(pending["pending"]) > 0:
        req_id = pending["pending"][0]["request_id"]
        # 3. Resolve it
        res_json = await resolve_review(req_id, "approve")
        res = json.loads(res_json)
        assert res["status"] == "completed"
        print(f"HITL test success: {req_id} resolved")
    else:
        pytest.fail("No pending request found for HITL test")

if __name__ == "__main__":
    # Manual run if pytest is not used
    async def run_all():
        print("Running MCP Bridge Verification...")
        try:
            await test_analyze_input()
            await test_secure_chat()
            await test_hitl_flow()
            print("All MCP Bridge tests passed successfully!")
        except Exception as e:
            print(f"Verification failed: {e}")

    asyncio.run(run_all())
