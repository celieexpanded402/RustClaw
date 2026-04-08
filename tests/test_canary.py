import requests

def test_canary():
    response = requests.get('https://api.example.com/endpoint')
    assert response.status_code == 200