import os
import dotenv
import tweepy
import requests

CHAR_LIMIT = 280

dotenv.load_dotenv('.env')

client = tweepy.Client(
    consumer_key=os.getenv("CONSUMER_KEY"),
    consumer_secret=os.getenv("CONSUMER_SECRET"),
    access_token=os.getenv("ACCESS_TOKEN"),
    access_token_secret=os.getenv("ACCESS_TOKEN_SECRET")
)

height, epoch, hashrate, supply = 'null', 'null', 'null', 'null'


# Pulling info from an API for now
base_url = 'https://bitcoinexplorer.org'


#block height
endpoint = '/api/blocks/tip/height/'
url = base_url + endpoint
try:
    response = requests.get(url, timeout=5); response.raise_for_status()
    height = response.json()
except requests.exceptions.HTTPError:
    pass


#epoch progress
epoch = height//210000
epoch_progress = (height%210000)/210000


#hashrate
endpoint = '/api/mining/hashrate/'
url = base_url + endpoint
try:
    response = requests.get(url, timeout=5); response.raise_for_status()
    data = response.json()
    hashrate = round(data['1Day']['val'], 2)
    hashrate_unit = 'EH/s'
except requests.exceptions.HTTPError:
    pass


#supply
endpoint = '/api/blockchain/coins/'
url = base_url + endpoint
try:
    response = requests.get(url, timeout=5); response.raise_for_status()
    supply = int(response.json())

except requests.exceptions.HTTPError:
    pass


#bitcoin holidays
endpoint = '/api/holidays/today?tzOffset=-3/'
url = base_url + endpoint
isHoliday = False
try:
    response = requests.get(url, timeout=5); response.raise_for_status()
    data = response.json()

    if len(data['holidays']):
        isHoliday = True
        holiday_name = data['holidays'][0]['name']
        holiday_desc = data['holidays'][0]['desc']
except requests.exceptions.HTTPError:
    pass


tweet = (f'height: {height}\n'
         f'hashrate: {hashrate} EH/s\n'
         f'supply: {supply:,}₿ [{supply/21000000:.2%}]\n'
         f'epoch: {epoch} [{epoch_progress:.2%}]')

if isHoliday:
    tweet += '\n\n' + 'Today is ' + holiday_name + '!'

    htmlLinkTag = '<a href='

    if htmlLinkTag in holiday_desc:
        pass
    else:
        if len(tweet + holiday_desc)<CHAR_LIMIT:
            tweet += '\n' + holiday_desc

print(tweet)

push = client.create_tweet(text=tweet)

