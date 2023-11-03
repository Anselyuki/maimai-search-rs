# image_operations.py

from PIL import Image, ImageDraw


def create_image():
    image = Image.new("RGB", (300, 300), "white")
    draw = ImageDraw.Draw(image)
    draw.text((10, 10), "Hello from PIL", fill="black")
    image.show()
