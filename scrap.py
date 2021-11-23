import requests
import bs4
import json

wiki_url = 'https://ffxiv.consolegameswiki.com/wiki/Main_Scenario_Quests'

def main():
    quest_info = []

    response = requests.get(wiki_url)
    response.raise_for_status()
    soup = bs4.BeautifulSoup(response.text, 'html.parser')
    for table in soup.find_all('table', class_='quest table'):
        section = table.find_previous_sibling().find('span')
        if not section:
            section = table.find_previous_sibling().find_previous_sibling().find('span')

        quests = []
        name = 'unknown'
        if section:
            name = section['id']

        tbody = list(table.children)[1]
        for row in tbody.find_all('tr'):
            items = list(row.children)
            quest_name = items[1].find('a')
            if quest_name:
                level = int(items[5].text)
                quests.append({
                    'name': quest_name['title'],
                    'level': level
                })

        quest_info.append({'name': name.replace('_', ' '), 'quests': quests})

    with open('quests.json', 'wt') as f:
        f.write(json.dumps(quest_info))

if __name__ == '__main__':
    main()