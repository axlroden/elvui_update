import os
import re
import time
import zipfile
import requests
import productdb_pb2

def query_api(apiurl):
    response = requests.get(apiurl)
    data = response.json()
    return data["version"], data["url"]

def local_version(folder):
    toc_loc = os.path.join(folder, 'ElvUI', 'ElvUI_Mainline.toc')
    try:
        with open(toc_loc, 'r') as toc:
            version = re.search(r'(?<=Version: ).*', toc.read()).group(0)
        return version
    except FileNotFoundError:
        return 'Not Installed'

def download_and_install(url, folder):
    local_filename = 'elvui.zip'
    req = requests.get(url, stream=True)
    with open(local_filename, 'wb') as download:
        for chunk in req.iter_content(chunk_size=1024):
            if chunk:
                download.write(chunk)
    with zipfile.ZipFile(local_filename, 'r') as elvui_zip:
        elvui_zip.extractall(path=folder)
    os.remove(local_filename)

def main():
    classic_era_path, classic_path, retail_path = installpath()
    prod, url = query_api('https://api.tukui.org/v1/addon/elvui')
    
    for path, label in [(classic_path, "classic"), (classic_era_path, "classic_era"), (retail_path, "retail")]:
        if path is not None:
            local = local_version(path)
            print(f'Installed {label} version: {local}')
            print(f'Current {label} version: {prod}')
            if local != prod:
                print(f'Updating {label}..')
                download_and_install(url, path)
                print(f'Update complete for {label}')

    url = "https://api.tukui.org/v1/changelog/elvui"
    text = requests.get(url).text
    get_changelog(text)
    time.sleep(60)

def get_changelog(changelog):
    lines = changelog.split('\n')

    found_version = False

    for line in lines:
        if line.startswith("## Version"):
            if found_version:
                break
            else:
                found_version = True
        if found_version:
            print(line.replace('\\', ''))

def installpath():
    productdb_path = os.path.join(os.getenv('ALLUSERSPROFILE'), 'Battle.net', 'Agent', 'product.db')
    paths = {}
    if os.path.exists(productdb_path):
        with open(productdb_path, "rb") as f:
            db = productdb_pb2.Database()
            db.ParseFromString(f.read())
            for entry in db.productInstall:
                code, installPath, extra = entry.productCode, entry.settings.installPath.replace("/", "\\"), entry.settings.productExtra
                paths[code] = os.path.join(installPath, extra, 'interface', 'addons')
    return paths.get('wow_classic_era'), paths.get('wow_classic'), paths.get('wow')

if __name__ == '__main__':
    main()
