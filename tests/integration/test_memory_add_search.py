from app.memory import Memory

def test_memory_add_search(memory):
    # Add a memory item with key 'test_key'
    memory.add('test_key', 'This is a test value')
    
    # Search for the added memory item by its key
    result = memory.search('test_key')
    assert result == ['This is a test value']