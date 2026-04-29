"""
 * Binary Search - Algorithm
 * 
 * This program implements binary search on a fixed sorted array of 10 integers.
 * It takes a target number as a command-line argument, searches for it using
 * binary search, and prints whether it was found and at which index.
 * 
 * Usage: python BinarySearch.py <number>
 *
"""

import sys

def binary_search(arr, target):

    low = 0
    high = len(arr) - 1
    
    while low <= high:
        mid = (low + high) // 2
        
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            low = mid + 1
        else:
            high = mid - 1
    
    return -1


def main():
    arr = [1, 3, 7, 12, 19, 25, 34, 41, 55, 78]
    
    if len(sys.argv) < 2:
        print("Usage: python program.py <number>")
        return 1
    
    try:
        target = int(sys.argv[1])
    except ValueError:
        print("Error: Please provide a valid integer")
        return 1
    
    result = binary_search(arr, target)
    
    if result >= 0:
        print(f"Found {target} at index {result}")
    else:
        print(f"{target} not found")
    
    return 0


if __name__ == "__main__":
    main()