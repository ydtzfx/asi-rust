from setuptools import setup, find_packages

setup(
    name="asi-sdk",
    version="0.1.0",
    description="Python SDK for the ASI AI Coding Assistant API",
    packages=find_packages(),
    install_requires=["requests>=2.28"],
    python_requires=">=3.9",
)
