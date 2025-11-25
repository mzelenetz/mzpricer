from setuptools import setup, Extension
import numpy

# Define the Cython extension
extensions = [
    Extension(
        "option_pricer", # The name of the resulting module
        ["option_pricer.pyx"],
        include_dirs=[numpy.get_include()],
        extra_compile_args=['-O3', '-ffast-math'], # Optimization flags
    )
]

setup(
    name='MZPricer',
    ext_modules=extensions,
    zip_safe=False,
)