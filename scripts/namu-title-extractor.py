import json
from time import sleep
from selenium import webdriver
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

startPage = 'https://namu.wiki/LongestPages'

driver = webdriver.Chrome()
driver.get(startPage)


def wait_for_load():
    wait = WebDriverWait(driver, 100)
    wait.until(EC.presence_of_element_located((By.CLASS_NAME, 'E8goUPurn')))


dumpCount = 0


def save_html():
    global dumpCount
    with open(f'dump/{dumpCount}.html', "w", encoding='UTF-8') as f:
        f.write(driver.page_source)
        dumpCount += 1


def extract_next_page_link():
    next_page_button = driver.find_element(
        By.CSS_SELECTOR, '#E8goUPurn > div.\\34 9e4646c > div > div > div > article > div:nth-child(6) > div > div:nth-child(4) > div > div > div > div > div > div > div > div > div:nth-child(2) > a:nth-child(2)')
    return next_page_button.get_attribute('href')


def extract_titles():
    table = driver.find_element(
        By.CSS_SELECTOR, '#VIVXXZe6L > div.\\34 9e4646c > div > div > div > article > div:nth-child(6) > div > div:nth-child(4) > div > div > div > div > div > div > div > div > ul ')
    elements = table.find_elements(
        By.CSS_SELECTOR, 'li')
    result = []
    for e in elements:
        # e.text is too slow, so save html and parse later is recommanded
        result.append(e.text)
    return result


page_link = startPage
result = []
while True:
    driver.get(page_link)
    # wait_for_load()
    sleep(1)
    # driver.delete_all_cookies()
    # result += extract_titles()
    page_link = extract_next_page_link()
    print(page_link)
    save_html()
    driver.delete_all_cookies()

    # with open(f'dump/{dumpCount}.txt', "w", encoding='UTF-8') as f:
    #     f.write(json.dumps(result, ensure_ascii=False))
    #     dumpCount += 1

    # sleep(50)
