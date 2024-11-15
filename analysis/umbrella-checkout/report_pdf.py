import os
from PIL import Image
from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import letter

# Define the directory with the PNG files and output PDF path
img_dir = "/home/gaps/tof-data/umbrella-checkout-mcmurdo/plots/6"
output_pdf = "/home/gaps/tof-data/umbrella-checkout-mcmurdo/plots/run6.pdf"

# Group files by integer identifier (e.g., 68) and side (A or B)
img_files = os.listdir(img_dir)
img_groups = {}
for file_name in img_files:
    if file_name.endswith((".png", ".webp")): 
        # Extract the integer identifier and side from filename
        parts = file_name.split(".")[0]
        num = None
        for i, char in enumerate(parts):
            if char.isdigit():
                num = ''.join(filter(str.isdigit, parts[i:]))
                side = parts[i + len(num):]
                break

        # If no number is found, assign a default key
        key = (int(num) if num else "no_number", side if num else "")
        img_groups.setdefault(key, []).append(os.path.join(img_dir, file_name))

def sort_key(item):
    num, side = item
    return (num if isinstance(num, int) else float('inf'), side)

# Sort groups by integer (or "no_number") and side
sorted_keys = sorted(img_groups.keys(), key=sort_key)


# Create the PDF and add each image group to it
c = canvas.Canvas(output_pdf, pagesize=letter)
width, height = letter

for key in sorted_keys:
    for img_path in sorted(img_groups[key]):
        img = Image.open(img_path)
        img_width, img_height = img.size
        aspect = img_width / img_height
        img.thumbnail((width, height))  # Resize to fit page if needed
        c.drawImage(img_path, 0, height - img.height, width=width, height=img.height)
        c.showPage()  # Add a new page after each image

c.save()
print(f"PDF created successfully at {output_pdf}")
